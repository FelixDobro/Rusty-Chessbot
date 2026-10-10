from concurrent.futures import ProcessPoolExecutor
from functools import partial
import chess
from sqlite_utils.db import Database
from tqdm import tqdm



DB_PATH = "../chess.db"
TABLE_NAME = "big_data"
CHUNK_SIZE = 100_000
NUM_WORKERS = 12


def clean(fen):
    splitted = fen.split(" ")
    splitted[4] = "0"
    splitted[5] = "0"
    return " ".join(splitted)

def flip_res(res):
    if res == "WhiteWin":
        return "BlackWin"
    elif res == "BlackWin":
        return "WhiteWin"
    else:
        return "Draw"
    
def process_chunks(offset, db_path, table_name, chunk_size):
    db = Database(db_path)
    
    query = f"SELECT fen, eval, res FROM {table_name} LIMIT {chunk_size} OFFSET {offset}"
    
    flipped_rows = []
    for row in db.execute(query):
        og_fen = row[0]
        board = chess.Board(og_fen).apply_transform(chess.flip_horizontal)
        
        flipped_rows.append({
            "fen": clean(board.fen()),
            "eval": row[1],
            "res": flip_res(row[2])
        })
    db.close()
    return flipped_rows

def augment(db_path, table_name, chunk_size):
    AUGMENT_STR = "_augmented"
    db = Database(db_path)
    nrows = db.execute(f"SELECT MAX(rowid) FROM {table_name}").fetchone()[0] or 0
    print("Dropping table")
    db[f"{table_name}{AUGMENT_STR}"].drop(ignore=True)
    print("Dropped old table")
    
    query = f"CREATE TABLE IF NOT EXISTS {table_name}{AUGMENT_STR} AS SELECT * FROM {table_name} WHERE 0"
    db.execute(query)
    db.commit()
    print("Created new one")

    offsets = list(range(0, nrows, chunk_size))
    worker_func = partial(
        process_chunks,
        db_path=db_path,
        table_name=table_name,
        chunk_size=chunk_size
    )
    

    offsets = list(range(0, nrows, CHUNK_SIZE))
    print(f"Found {nrows} rows")
    with ProcessPoolExecutor(max_workers=NUM_WORKERS) as executor:
        with tqdm(total=len(offsets)) as pbar:
            for i in range(0, len(offsets), NUM_WORKERS):
                batch_offsets = offsets[i : i + NUM_WORKERS]
                for res in executor.map(worker_func, batch_offsets):
                    db[f"{table_name}{AUGMENT_STR}"].insert_all(res, ignore=True)
                    pbar.update(1)


if __name__ == "__main__":

    augment(DB_PATH, TABLE_NAME, CHUNK_SIZE)