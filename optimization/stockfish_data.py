import os
import chess
from dotenv import load_dotenv
from datasets import load_dataset, Dataset, Features, Value, Sequence, enable_progress_bar
from encode import encode
from huggingface_dataset import scale
from data import is_quiet

MATE_CP_VAL = 32000
BATCH_SIZE = 50_000
WORKERS = 12
REVISION = "stockfish_run_1"

def generator():
    streamed_ds = load_dataset(
        "Lichess/chess-position-evaluations",
        split="train",
        streaming=True,
        token=os.getenv("HUGGING_FACE_TOKEN")
    )
    
    for batch in streamed_ds.iter(batch_size=BATCH_SIZE):
        fens = batch["fen"]
        cp_vals = batch["cp"]

        for fen, cp_raw in zip(fens, cp_vals):
            board = chess.Board(fen)
            if not is_quiet(board):
                continue

            own, opp = encode(board)
            cp = cp_raw if cp_raw is not None else MATE_CP_VAL
            eval_score = float(scale(cp))

            yield {
                "Own_pov": own,
                "Opp_pov": opp,
                "Win_prob": eval_score,
                "Outcome": 1.0
            }


if __name__ == "__main__":
    load_dotenv()
    token = os.getenv("HUGGING_FACE_TOKEN")
    if not token:
        raise ValueError("No token provided")

    schema = Features({
        "Own_pov": Sequence(Value("int32")),
        "Opp_pov": Sequence(Value("int32")),
        "Win_prob": Value("float32"),
        "Outcome": Value("float32")
    })

    enable_progress_bar()
    dataset = Dataset.from_generator(generator, features=schema, num_proc=WORKERS)
    dataset.push_to_hub(
        "FelixDobro/chess-training-encodet",
        token=token,
        private=False,
        revision=REVISION
    )