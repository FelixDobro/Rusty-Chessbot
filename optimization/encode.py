from functools import partial

from sqlite_utils.db import Database
from itertools import repeat

from concurrent.futures import ProcessPoolExecutor  
import chess
import numpy as np
from tqdm import tqdm


DB_PATH = "../chess.db"
EVAL_TABLE_NAME = "big_data"
TRAIN_TABLE_NAME = "training"
LAMBDA = 0.95
CHUNK_SIZE = 100_000
CP_SCALE = 400
NUM_WORKERS = 12


offset = [0,1,2,3,4]
OPPONENT_OFFSET = 5
not_king = [chess.PAWN, chess.KNIGHT, chess.BISHOP, chess.ROOK, chess.QUEEN]

def flip(sq):
    return sq ^ 56

def encode(board: chess.Board) -> tuple[np.ndarray, np.ndarray]:
    wk_pos = board.pieces(chess.KING, chess.WHITE).pop()
    bk_pos = board.pieces(chess.KING, chess.BLACK).pop()
    wk_offset = wk_pos * 11 * 64 
    bk_offset = flip(bk_pos) * 11 * 64
    w_indices = []
    b_indices = []
    w_indices.append(wk_offset + 10 * 64 + bk_pos)
    b_indices.append(bk_offset + 10 * 64 + flip(wk_pos))
    for piece in not_king:
        for sq in board.pieces(piece, chess.WHITE):
            idx = piece - 1
            wp_offset = 64 * offset[idx] + sq
            bp_offset = 64 * (OPPONENT_OFFSET + offset[idx]) + flip(sq)
            w_indices.append(wk_offset + wp_offset)
            b_indices.append(bk_offset + bp_offset)
        for sq in board.pieces(piece, chess.BLACK):
            idx = piece - 1
            wp_offset = 64 * (OPPONENT_OFFSET + offset[idx]) + sq
            bp_offset = 64 * offset[idx] + flip(sq)
            w_indices.append(wk_offset + wp_offset)
            b_indices.append(bk_offset + bp_offset)
    w = np.array(w_indices)
    b = np.array(b_indices)

    return [w,b] if board.turn else [b,w]



def game_outcome(res, turn):
    if res == "Draw":
        return 0.5
    if res == "BlackWin":
        return 0 if turn else 1
    else:
        return 1 if turn else 0

def sigmoid(x):
    return 1 / (1 + np.exp(-x))

def prepare_for_training(offset, db_path, eval_table, chunk_size):
    db = Database(db_path)
    rows = db[eval_table].rows_where(f"id >= {offset} AND id < {offset + chunk_size}")
    res = []
    for row in rows:
        
        board = chess.Board(row["fen"])
        own, opp = encode(board)
        eval_scaled = sigmoid(row["eval"] / CP_SCALE)
        if eval_scaled > 1:
            print(f"Eval scaled too big: {eval_scaled}")
        outcome = game_outcome(row["res"], board.turn)
        target_eval = LAMBDA * eval_scaled + (1.0 - LAMBDA) * outcome
        info = {
            "own_pov": own.tobytes(),
            "opp_pov": opp.tobytes(),
            "eval": target_eval,
    
        }
        res.append(info)
    db.close()
    return res


if __name__ == "__main__":
    

    db = Database(DB_PATH)
    db.enable_wal()

    print("Creating new DB for training data")
    new_db = Database("training_db")
    print("DB created")
    new_db[TRAIN_TABLE_NAME].drop(ignore=True)
    new_db.create_table(ignore=True, name=TRAIN_TABLE_NAME,columns={
        "id": int,
        "own_pov": bytes,
        "opp_pov": bytes,
        "eval": float,

    },pk="id",
    not_null=["own_pov", "opp_pov", "eval"]
    )
    new_db.enable_wal()
    nrows = db.execute(f"SELECT MAX(rowid) FROM {EVAL_TABLE_NAME}").fetchone()[0] or 0
    
    worker_func = partial(
        prepare_for_training,
        db_path=DB_PATH,
        eval_table=EVAL_TABLE_NAME,
        chunk_size=CHUNK_SIZE
    )
    db.close()
    
    offsets = list(range(0, nrows, CHUNK_SIZE))
    print(f"Found {nrows} rows")
    with ProcessPoolExecutor(max_workers=NUM_WORKERS) as executor:
        with tqdm(total=len(offsets)) as pbar:
            for i in range(0, len(offsets), NUM_WORKERS):
                batch_offsets = offsets[i : i + NUM_WORKERS]
                for res in executor.map(worker_func, batch_offsets):
                    new_db[TRAIN_TABLE_NAME].insert_all(res)
                    pbar.update(1)
    new_db.close()