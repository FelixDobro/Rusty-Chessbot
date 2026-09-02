import chess.polyglot
from tensorboardX import SummaryWriter
import os
import json
from chess import engine
import ray
import chess
import time
import numpy as np
from chess.engine import SimpleEngine
from tqdm import tqdm



CONCURRENCY = 10
HASH_SIZE = 100    # Warning: 12 * 2 * HASH_SIZE is the number of mb used for training
ENGINE_CMD = "./engines/unoptimized"
RUN = "FEW_GAMES_PER_ITER"
NUM_OPENING_MOVES = 10
NUM_ITERATIONS = 5_000
GAMES_PER_ITERATION = 5 # * 2 because every opening is played twice
NODES = 50_000

VERBOSE = os.environ.get("verbose", True)
RESULT_UPDATE = 25

cur_iteration = 0


class Player:
    def __init__(self):
        self.engine = SimpleEngine.popen_uci(ENGINE_CMD)
        self.engine.protocol.send_line(f"setoption name Hash value {HASH_SIZE}")
        self.engine.ping()
       
    def set_options(self, values, names):
      
        self.engine.configure({names[i]: values[i] for i in range(len(values))})
        self.engine.ping()

    def close(self):
        self.engine.close()
        
    def reset(self):
        self.engine.protocol.send_line("ucinewgame")
        self.engine.ping()


@ray.remote
class MatchMaker:

    def __init__(self):
        self.pos = Player()
        self.neg = Player()

    def change_params(self, pos: np.ndarray, neg: np.ndarray, names: np.ndarray):
        self.pos.set_options(pos, names)
        self.neg.set_options(neg, names)

    def shutdown(self):
        self.pos.close()
        self.neg.close()

    def play_game(self, board: chess.Board, pos_white: bool):
        white = self.pos if pos_white else self.neg
        black = self.pos if white == self.neg else self.neg

        white.reset()
        black.reset()

        while not board.is_game_over(claim_draw=True):

            cur = white if board.turn else black            
            #
            res = cur.engine.play(board, engine.Limit(nodes=NODES
            ))
            
            board.push(res.move)

        return self.__parse_result(board, white)

    def __parse_result(self, board, white):
        # "1-0", "0-1", "1/2-1/2"
        res = board.result(claim_draw=True)
        if res == "1/2-1/2":
            return 0
        if res == "1-0":
            return 1 if self.pos == white else -1
        if res == "0-1":
            return 1 if self.neg == white else -1
        raise("ValueError")



def board_with_opening() -> chess.Board:
    board = chess.Board()
    with chess.polyglot.open_reader("uho-pohl.bin") as reader:
        for _ in range(NUM_OPENING_MOVES):
            try:
                entry = reader.weighted_choice(board)
                move = entry.move
                board.push(move)
            except IndexError:
                break
        return board
    
def stochastic_round(array: np.ndarray) -> np.ndarray:

    floor = np.floor(array)
    probs = array - floor
    randoms = np.random.random(*array.shape)
    floor += probs > randoms
    return floor


class SPSA:

    def __init__(self):
        
        self.start = None
        os.makedirs(f"logs/{RUN}", exist_ok=True)
        self.logger = SummaryWriter(logdir=f"logs/{RUN}")
        engine = SimpleEngine.popen_uci(ENGINE_CMD)
        param_min = []
        param_max = []
        param_cur = []
        param_name = []

        for name, param in engine.options.items():
            if param.type == "spin":
                param_min.append(param.min)
                param_max.append(param.max)
                param_cur.append((param.default - param.min) / (param.max - param.min))
                param_name.append(name)

        self.mins = np.array(param_min, dtype = np.float32)
        self.maxs = np.array(param_max, dtype = np.float32)
        self.cur = np.array(param_cur, dtype = np.float32)
        self.name = np.array(param_name)
        self.initial = self.cur.copy()

        engine.close()
        self.workers = [MatchMaker.remote() for _ in range(CONCURRENCY)]
        if VERBOSE:
            print(f"Found {len(param_name)} spin parameters to optimize")

    def result_file(self, step):
        if VERBOSE:
            print("Saving training data")
        end = time.monotonic()
        duration = end - self.start
        hours = int(duration // 60 // 60)
        duration -= hours * 60 ** 2
        minutes = int(duration // 60)
        duration -= minutes * 60
        seconds = int(duration)

        final_params = self.scale_up(self.cur)
        initial_params = self.scale_up(self.initial)
        just_final = {}
        param_dict = {}
        for i in range(len(final_params)):
            before = round(initial_params[i], ndigits=2)
            after = round(final_params[i], ndigits=2)

            param_dict[self.name[i]] = f"Before: {before}, After: {after}, Change: {round(after - before)}"
            just_final[self.name[i]] = after

        data = {
            "metadata": {
                "run_name": RUN,
                "Time": f"{hours}h {minutes}min {seconds}s",
                "iterations": step,
                "concurrency": CONCURRENCY,
                "hash_size_mb": HASH_SIZE,
                "npm": NODES 
            },
            "Parameters": param_dict,
            "Final": just_final
        }
        os.makedirs("results", exist_ok=True)
        with open(f"results/{RUN}.json", mode="w") as f:
            json.dump(
                data, 
                f, 
                indent=4, 
                default=lambda x: x.item() if hasattr(x, "item") else str(x)
            )



    def scale_up(self, vec):
        return vec * (self.maxs - self.mins) + self.mins

    def scale_down(self, vec):
        return (vec - self.mins) / (self.maxs - self.mins)

    def scale_and_round(self, vec):
        scaled = self.scale_up(vec)
        rounded = stochastic_round(scaled)
        return rounded.clip(min=self.mins, max= self.maxs)

    def pertubated(self, ck):
        random = np.random.random(*self.cur.shape)
        one_indices = random > 0.5
        ones = np.ones_like(random)
        sign = np.where(one_indices, ones, - ones)
        pertubation = sign * ck
        pos = self.cur + pertubation
        neg = self.cur - pertubation
        return self.scale_and_round(pos), self.scale_and_round(neg)


    def play_round(self):
        executing = {}
        boards = [(board_with_opening(), True) for _ in range(GAMES_PER_ITERATION)]
        boards = boards + [(b.copy(), False) for b,_ in boards]
        for worker in self.workers:
            if not boards:
                break
            board, pos = boards.pop()
            future = worker.play_game.remote(board, pos)
            executing[future] = worker

        res_sum = 0

        while executing:
            ready, _ = ray.wait(list(executing.keys()), num_returns=1)
            finished = ready[0]

            free_worker = executing.pop(finished)
            res_sum += ray.get(finished)

            if boards:
                board, pos = boards.pop()
                future = free_worker.play_game.remote(board, pos)
                executing[future] = free_worker

        return res_sum / (GAMES_PER_ITERATION * 2)

    def log(self, step, ak, ck):
        scaled = self.scale_up(self.cur)
        for i in range(len(self.cur)):
            self.logger.add_scalars(
                main_tag=self.name[i],
                tag_scalar_dict={
                    "current": scaled[i],
                    "min": self.mins[i],
                    "max": self.maxs[i],
                },
                global_step=step
            )
        self.logger.add_scalar("hyperparams/ak", ak, step)
        self.logger.add_scalar("hyperparams/ck", ck, step)
        self.logger.flush()

    def optimize(self):
        global cur_iteration

        self.start = time.monotonic()
        A = 0.1 * NUM_ITERATIONS
        alpha = 0.602
        gamma = 0.101
        a = 0.06
        c = 0.08   

        for i in tqdm(range(NUM_ITERATIONS)):
            cur_iteration = i
            ak = a / (A+ i + 1 ) ** alpha
            ck = c / (i + 1) ** gamma

            pos, neg = self.pertubated(ck)
            futures = [worker.change_params.remote(pos, neg, self.name) for worker in self.workers]
            ray.get(futures)

            dy = self.play_round()
            dx = self.scale_down(pos) - self.scale_down(neg)
            new = ak * np.divide(dy, dx, out=np.zeros_like(dx), where=dx!=0)
            self.cur = np.clip(self.cur + new, 0, 1)

            self.log(i, ak, ck)
            if VERBOSE:
                if i % RESULT_UPDATE == 0: 
                    self.result_file(i)
                print(f"Win ratio: {dy}")

        ray.get([worker.shutdown.remote() for worker in self.workers])
        self.logger.close()
        self.result_file(NUM_ITERATIONS)



if __name__ == "__main__":

    spsa = SPSA()
    try:
        spsa.optimize()
    except KeyboardInterrupt:
        spsa.result_file(cur_iteration)
    except chess.engine.EngineTerminatedError:
        spsa.result_file(cur_iteration)