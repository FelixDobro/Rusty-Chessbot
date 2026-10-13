from encode import encode, sigmoid, game_outcome
from sqlite_utils import Database
import chess
import math
from dotenv import load_dotenv
import os
from datasets import Features, Value, Sequence, Dataset, enable_progress_bar

DB_PATH = "../chess.db"
EVAL_TABLE_NAME = "big_data"
REVISION = "training_run_1"
CHUNK_SIZE = 10000
NUM_PROC = 12
CHUNK_SIZE = 10000


def scale(cp_val, scale=400):
    return sigmoid(cp_val / scale)

def generator(chunks_for_this_worker):
    db = Database(DB_PATH)
    cursor = db.conn.cursor()

    for start_id, end_id in chunks_for_this_worker:
     
        query = f"""
            SELECT fen, eval, res 
            FROM {EVAL_TABLE_NAME} 
            WHERE rowid >= ? AND rowid < ?
        """
        cursor.execute(query, (start_id, end_id))

        while True:
            res = cursor.fetchmany(CHUNK_SIZE)
            if not res:
                break
            for r in res:
                board = chess.Board(r[0])
                own_pov, opp_pov = encode(board)
                eval = scale(r[1])
                outcome = game_outcome(r[2], board.turn)

                out = {
                    "Own_pov": own_pov,
                    "Opp_pov": opp_pov,
                    "Win_prob": eval,
                    "Outcome": outcome
                }
                yield out

    cursor.close()
    db.close()


def chunks():


    db = Database(DB_PATH)
    db.enable_wal()
    cursor = db.conn.cursor()

    cursor.execute(f"SELECT MIN(rowid), MAX(rowid) FROM {EVAL_TABLE_NAME}")
    min_id, max_id = cursor.fetchone()
    cursor.close()
    db.close()

    id_range = max_id - min_id + 1
    range_per_worker = math.ceil(id_range / NUM_PROC)

    db_chunks = []
    for i in range(NUM_PROC):
        start_id = min_id + (i * range_per_worker)
        end_id = start_id + range_per_worker
        db_chunks.append((start_id, end_id))
    

    return db_chunks



if __name__ == "__main__":

    load_dotenv()
    token = os.getenv("HUGGING_FACE_TOKEN")
    if not token:
        raise ValueError("No token provided")
    print("Found token")
    schema = Features({
        "Own_pov": Sequence(Value("int64")),
        "Opp_pov": Sequence(Value("int64")),
        "Win_prob": Value("float32"),
        "Outcome": Value("float32")
    })

    print("Generating Dataset")
    enable_progress_bar()
    dataset = Dataset.from_generator(
        generator,
        features=schema,
        gen_kwargs={"chunks_for_this_worker": chunks()},
        num_proc=NUM_PROC
    )
    print("Uploading Dataset to hugging face")
    dataset.push_to_hub("FelixDobro/chess-training-encodet", token=token, private=False, revision=REVISION)