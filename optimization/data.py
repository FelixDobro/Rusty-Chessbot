import chess
from chess.engine import SimpleEngine, Limit
from sqlite_utils.db import Database
from tqdm import tqdm
from itertools import repeat
from concurrent.futures import ProcessPoolExecutor

ALREADY_EVALUATED = "_already_evaluated"


def clean(fen):
    splitted = fen.split(" ")
    splitted[4] = "0"
    splitted[5] = "0"
    return " ".join(splitted)


def is_quiet(board: chess.Board) -> bool:

    if board.is_check():
        return False
    for move in board.legal_moves:
        if board.is_capture(move):
            return False

        if move.promotion is not None:
            return False

    return True

b = chess.Board()
assert(is_quiet(b))
b.push(move=chess.Move.from_uci("e2e4"))
assert(is_quiet(b))
b.push(move=chess.Move.from_uci("d7d5"))
assert(not is_quiet(b))
b.push(move=chess.Move.from_uci("e4e5"))
assert(is_quiet(b))
b.pop()
b.push(move=chess.Move.from_uci("f1b5"))
assert(not is_quiet(b))
b.push(move=chess.Move.from_uci("c7c6"))

from chess.engine import SimpleEngine, Limit
from sqlite_utils.db import Database
from tqdm import tqdm
from itertools import repeat
from concurrent.futures import ProcessPoolExecutor


ALREADY_EVALUATED = "_already_evaluated"


def parse_result(result_str: str) -> str:
    outcomes = {
        "1-0": "WhiteWin",
        "0-1": "BlackWin",
        "1/2-1/2": "Draw"
    }
    return outcomes.get(result_str, "Draw")

def play_game_and_save(considered, fens, engine_path, nodes):
    
    engine = SimpleEngine.popen_uci(engine_path)
    engine.configure({"Hash": 5})        

    positions = []
    
    for fen in fens:
        board = chess.Board(fen)
        engine.protocol.send_line("ucinewgame")
        try:
            while not board.is_game_over(claim_draw=True):
                clean_fen = clean(board.fen())
            
                res = engine.analyse(board, limit=Limit(nodes=nodes))
                
                if "pv" not in res or not res["pv"]:
                    print(f"\n[Fehler] Engine lieferte keinen Zug für FEN: {board.fen()}")
                    break
                    
                if is_quiet(board):

                    if "score" in res and res["score"] is not None:
                        positions.append({
                            "fen": clean_fen,
                            "eval": res["score"].pov(board.turn).score()
                        })
                    
                board.push(res["pv"][0])

        except Exception as e:
            print(f"\n[Crash] Fehler beim Spielen von FEN {fen}: {e}")
            engine.close()
            engine = SimpleEngine.popen_uci(engine_path)
            continue

        outcome = parse_result(board.result(claim_draw=True))
        for pos in positions:
            pos["res"] = outcome
    
    engine.close()
    
    return considered, positions

def evaluate(engine: str, db_path: str, table_name:str, nodes=10_000, chunk_size=10):
    db = Database(db_path)
    db.enable_wal()
    db.create_table(ignore=True, name=table_name,columns={
        "id": int,
        "fen": str,
        "eval": float,
        "res": str
    },pk="id",
    not_null=["fen", "eval", "res"]
    )
    db[table_name].create_index(["fen"], unique=True, if_not_exists=True)

    db.create_table(ignore=True, name=f"{table_name}{ALREADY_EVALUATED}",columns={
        "id": int,
        "pos_id": int,
    },pk="id",
    not_null=["pos_id"],
    foreign_keys=[("pos_id", "positions", "id")]
    )

    fens = []
    ids = []
    cur_fens = []
    cur_ids = []
    num_rows = 0
    for row in db["positions"].rows_where(f"id NOT IN (SELECT pos_id FROM {table_name}{ALREADY_EVALUATED})", select="id, fen"):
        num_rows += 1
        if len(cur_fens) == chunk_size:
            fens.append(cur_fens)
            ids.append(cur_ids)
            cur_ids = []
            cur_fens = []
        cur_fens.append(row["fen"])
        cur_ids.append({"pos_id": row["id"]})

    print(f"Found {num_rows} positions to evaluate")
    with ProcessPoolExecutor(max_workers=18) as executor:
        for considered, positions in tqdm(executor.map(play_game_and_save, ids, fens, repeat(engine), repeat(nodes)), total=len(ids)):
            db[table_name].insert_all(positions, ignore=True)
            db[f"{table_name}{ALREADY_EVALUATED}"].insert_all(considered, ignore=True)
    
    db.close()


if __name__ == "__main__":

    evaluate("./engines/pesto", db_path="../chess.db", table_name="big_data", chunk_size=15, nodes=10_000)