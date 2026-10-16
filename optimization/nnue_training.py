import os

from datasets import Dataset, load_dataset
from dotenv import load_dotenv
import numpy as np
import torch
import gzip
from torch.nn.utils.rnn import pad_sequence
from torch.optim import AdamW
from torch.optim.lr_scheduler import CosineAnnealingLR
from torch.nn import functional as F
from tqdm import tqdm
from torch.utils.data.dataloader import DataLoader
from torch import nn
import math


DEVICE = torch.device("cuda" if torch.cuda.is_available() else "cpu")
DATA_LOADER_WORKERS = 12
EVALUATION_SIZE = 0.01  # What portion of data will be used for evaluation 
EPOCHS = 30
CP_SCALE = 400
LAMBDA = 1
BATCH_SIZE = 2**15
REPO_NAME = "FelixDobro/chess-training-encodet"
REVISION = "stockfish_run_1"   
LR = 1e-3
WEIGHT_DECAY = 1e-5



def prev_power_two(x: float) -> float: 
    _, exponent = math.frexp(x)
    
    return math.ldexp(1.0, exponent - 1)


class SCReLU(nn.Module):
    def __init__(self, max=1):
        super().__init__()
        self.max = max

    def forward(self, x):
        x = torch.clamp(x, 0, self.max)
        return x ** 2

def quantize_ste(w, scale, min_val, max_val):
    scaled = w * scale
    w_clamped = torch.clamp(scaled, min_val, max_val)
    w_int = torch.round(w_clamped)
    w_quant = w_int / scale
    return w + (w_quant - w).detach()


class SCReLU_ste(nn.Module):
    def __init__(self, scale, div):
        super().__init__()
        self.scale = scale
        self.div = div
     

    def forward(self, x):
        x_int = torch.round(x * self.scale)
        clamped = torch.clamp(x_int, 0.0, self.scale - 1)
        squared = clamped * clamped
        shifted = torch.floor(squared / self.div)
        
        x_quant = shifted / QA

        x_float = (torch.clamp(x, 0.0, (self.scale - 1) / self.scale) ** 2) 
        return x_float + (x_quant - x_float).detach()

    
QA = 256
QB = 64
PAD_IDX = 11 * 64 * 64
class Accumulator(nn.Module):
    
    def __init__(self):
        super().__init__()
        
        EMBEDDINGS = 11*64*64 + 1
        L1_SIZE = 1024
        self.padding_idx = PAD_IDX
        self.weights = nn.Parameter(torch.randn(EMBEDDINGS, L1_SIZE) * 0.05)
        self.bias = nn.Parameter(torch.zeros(L1_SIZE))

    def forward(self, x):
        # 1. ZUERST die kleine Gewichtematrix (184 MB) quantisieren, NICHT den 4GB Batch!
        w_quant = quantize_ste(self.weights, QA, -2**15, 2**15 - 1)
        b_quant = quantize_ste(self.bias, QA, -2**15, 2**15 - 1)

        # 2. F.embedding_bag summiert direkt in C++/CUDA über dim=1, 
        # OHNE den 4.16 GB großen [Batch, Pieces, 1024] Tensor im VRAM anzulegen!
        return F.embedding_bag(x, w_quant, mode="sum", padding_idx=self.padding_idx) + b_quant

class Hidden(nn.Module):
    def __init__(self, scale, min, max, IN, OUT):
        super().__init__()
        self.scale = scale
        self.min = min
        self.max = max
        stdv = 1.0 / math.sqrt(IN)
        self.weight = nn.Parameter(torch.randn((OUT, IN)) * 0.05)
        self.bias = nn.Parameter(torch.zeros(OUT))

    def forward(self, x):
        w = quantize_ste(self.weight, self.scale, min_val=self.min, max_val=self.max)
        b = quantize_ste(self.bias, QA * QB, min_val=-2**31, max_val=2**31 - 1)

        return F.linear(x, w, b)

class NNUE(nn.Module):
    def __init__(self):
        super().__init__()
        

        self.accumulator = Accumulator()

        self.hidden = nn.Sequential(
            SCReLU_ste(QA, QA),
            Hidden(QB, min=-128, max=127, IN=2048, OUT=8),
            SCReLU_ste(QA * QB, div=QA*QB*QB),
            Hidden(QB, min=-128, max=127, IN=8, OUT=32),
            SCReLU_ste(QA * QB, div=QA*QB*QB),
            Hidden(QB, min=-128, max=127, IN=32, OUT=1),
        )



    def forward(self, cur_indices, opp_indices):

        cur_embedding = self.accumulator(cur_indices) 
        opp_embedding = self.accumulator(opp_indices)
        out_raw = self.hidden(torch.cat([cur_embedding, opp_embedding], dim=-1))

        return out_raw

    def save_quantized(self, dir: str):

        os.makedirs(dir, exist_ok=True)
        file_dir = f"{dir}/nnue_weights.bin" 
    
        quantized_accum = (self.accumulator.weights[:PAD_IDX,:] * QA).detach().cpu().numpy().round().clip(-2**15,2**15-1)
        quantized_accum_bias = (self.accumulator.bias * QA).detach().cpu().numpy().round().clip(-2**15,2**15-1)
        assert np.abs(quantized_accum).max() < 2**15, "Quantized value does not fit into i16"
        assert np.abs(quantized_accum_bias).max() < 2**15, "Quantized value does not fit into i16"

        total = 0
        with gzip.GzipFile(file_dir, mode="wb") as f:
            print(f"Accumulator shape: {quantized_accum.shape}, multiplied with QA")
            quantized_accum = quantized_accum.astype(np.int16)
            f.write(quantized_accum.tobytes())
            total += quantized_accum.nbytes
            print(f"Accumulator bias shape: {quantized_accum_bias.shape}, multiplied with: QA, fits into i16")
            quantized_accum_bias = quantized_accum_bias.astype(np.int16)
            f.write(quantized_accum_bias.tobytes())
            total += quantized_accum_bias.nbytes
            print("Saved accum")
            for param, weight in self.hidden.named_parameters():
                print(f"Saving {param}")
                if "bias" in param:
                    quantized = (weight * QB * QA).detach().cpu().numpy().round()
                    if "1.bias" in param:
                        quantized = np.pad(quantized, (0,8), mode="constant", constant_values=0)
                    if "5.bias" in param:
                        quantized = np.pad(quantized, (0, 15), mode="constant", constant_values=0)
                    assert np.abs(quantized).max() <= 2**31, "Quantized value does not fit into i32"
                    quantized = quantized.astype(np.int32)
                    f.write(quantized.tobytes())
                    total += quantized.nbytes
                    print(f"{param} shape: {quantized.shape}, multiplied with: QA * QB, fits into i32")
                else:
                    quantized = (weight * QB).detach().cpu().numpy().round().clip(- 2**7, 2**7 - 1)
                    if "3.weight" in param:
                        quantized = quantized.transpose()
                    if "5.weight" in param:
                        quantized = np.pad(quantized, ((0,0),(0, 32)), mode="constant", constant_values=0)
                                            
                    assert np.abs(quantized).max() <= 2**7, "Quantized value does not fit into i8"
                    quantized = quantized.astype(np.int8)
                    f.write(quantized.tobytes())
                    total += quantized.nbytes
                    print(f"{param} shape: {quantized.shape}, multiplied with: QB, fits into i8")
            
        print(f"Quantized with values QA: {QA} and QB: {QB}, total bytes: {total}, % 64 = {total % 64}")





class Positions(Dataset):
    def __init__(self, db_path, table_name):
        super().__init__()

        self.db = None
        self.db_path = db_path
        self.table_name = table_name

        

    def __getitems__(self, indices):
        if self.db == None:
            self.db = Database(self.db_path)

        indices = [idx + 1 for idx in indices]
        place_holder = ",".join("?" for _ in range(len(indices)))

        query = f"SELECT own_pov, opp_pov, eval FROM {self.table_name} WHERE id IN ({place_holder})"
        own, opp, eval = [], [], []
        for row in self.db.execute(query, indices):
            own.append(row[0])
            opp.append(row[1])
            eval.append(row[2])

        own_tensors = [torch.from_numpy(np.frombuffer(b, dtype=np.int64).copy()) for b in own]
        opp_tensors = [torch.from_numpy(np.frombuffer(b, dtype=np.int64).copy()) for b in opp]
                
        own_padded = pad_sequence(own_tensors, batch_first=True, padding_value=PAD_IDX)
        opp_padded = pad_sequence(opp_tensors, batch_first=True, padding_value=PAD_IDX)

        
        eval_tensor = torch.tensor(eval, dtype=torch.float32).unsqueeze(1)

        return own_padded, opp_padded, eval_tensor
    
    def __len__(self):
        temp_db = Database(self.db_path)
        count = temp_db.execute(f"SELECT MAX(rowid) FROM {TRAIN_TABLE_NAME}").fetchone()[0] or 0
        temp_db.conn.close()
        return count



def collate_fn(batch):
    own_povs = [item["Own_pov"] for item in batch]
    opp_povs = [item["Opp_pov"] for item in batch]
    win_probs = [item["Win_prob"] for item in batch]
    outcomes = [item["Outcome"] for item in batch]

    own = pad_sequence(own_povs, batch_first=True, padding_value=PAD_IDX)
    opp = pad_sequence(opp_povs, batch_first=True, padding_value=PAD_IDX)
    win_probs = torch.stack(win_probs)
    outcomes = torch.stack(outcomes)
    targets = win_probs * LAMBDA + outcomes * (1-LAMBDA)
    return own, opp, targets.unsqueeze(1)


def fast_collate_fn(batch_dict):
    own = pad_sequence(batch_dict["Own_pov"], batch_first=True, padding_value=PAD_IDX)
    opp = pad_sequence(batch_dict["Opp_pov"], batch_first=True, padding_value=PAD_IDX)
    
    win_probs = batch_dict["Win_prob"]
    outcomes  = batch_dict["Outcome"]
    
    targets = (win_probs * LAMBDA + outcomes * (1.0 - LAMBDA)).unsqueeze(1)
    return own, opp, targets


def identity_collate_fn(batch):
    return batch

if __name__ == "__main__":
    print("Loading data")
    os.makedirs("model_weights", exist_ok=True)
    load_dotenv()
    token = os.getenv("HUGGING_FACE_TOKEN")
    if not token:
        raise ValueError("No token provided")
    print("Found token")

    train_raw = load_dataset(REPO_NAME, token=token, revision=REVISION, num_proc=DATA_LOADER_WORKERS, split="train[:99%]")
    val_raw = load_dataset(REPO_NAME, token=token, revision=REVISION, num_proc=DATA_LOADER_WORKERS, split="train[99%:]")

    # 516 und 12 sind glatt durch 12 (DATA_LOADER_WORKERS) teilbar
    train_stream = train_raw.to_iterable_dataset(num_shards=720).shuffle(buffer_size=50_000, seed=42).with_format("torch").batch(BATCH_SIZE)
    val_stream   = val_raw.to_iterable_dataset(num_shards=24).with_format("torch").batch(BATCH_SIZE)

    train_loader = DataLoader(
        train_stream,
        num_workers=DATA_LOADER_WORKERS,
        batch_size=None,
        collate_fn=fast_collate_fn,
        pin_memory=True,
        prefetch_factor=2,  
    )
    test_loader = DataLoader(
        val_stream,
        num_workers=DATA_LOADER_WORKERS,
        collate_fn=fast_collate_fn,
        batch_size=None,
        pin_memory=True,
        prefetch_factor=2,
        persistent_workers=True
    )
    print("Data Loaded")
    
    net = NNUE().to(DEVICE)

    optim = AdamW(net.parameters(), lr=LR, weight_decay=WEIGHT_DECAY)
    scheduler = CosineAnnealingLR(optim, T_max=EPOCHS, eta_min=1e-6)
    print(f"Starting training on {DEVICE}")
    best_test_loss = float("inf")
    for i in range(EPOCHS):
        train_stream.set_epoch(i)
        print(f"\n--- Epoch {i+1}/{EPOCHS} | LR: {scheduler.get_last_lr()[0]:.6f} ---")
        
        net.train()
        epoch_loss = 0.0
        train_samples = 0
        
        pbar = tqdm(train_loader, desc="Train")
        for batch in pbar:
            current, opponent, eval = batch
            current = current.to(DEVICE, non_blocking=True)
            opponent = opponent.to(DEVICE, non_blocking=True)
            eval = eval.to(DEVICE, non_blocking=True)

            out = torch.sigmoid(net(current, opponent))
            loss = F.mse_loss(out, eval)
            
            optim.zero_grad(set_to_none=True)
            loss.backward()
            optim.step()
            
            batch_size = current.size(0)
            epoch_loss += loss.item()
            
        scheduler.step()
        
        net.eval()
        test_loss = 0.0
        
        with torch.no_grad():
            for batch in tqdm(test_loader, desc="Eval"):
                current, opponent, eval = batch
                current = current.to(DEVICE, non_blocking=True)
                opponent = opponent.to(DEVICE, non_blocking=True)
                eval = eval.to(DEVICE, non_blocking=True)

                out = torch.sigmoid(net(current, opponent))
                loss = F.mse_loss(out, eval)
                
                batch_size = current.size(0)
                test_loss += loss.item() 


        print(f"Train Loss: {epoch_loss:.6f} | Val Loss: {test_loss:.6f}")
        if test_loss < best_test_loss:
            print("Net params improved, saving weights")
            best_test_loss = test_loss
            torch.save(net.state_dict(), "model_weights/model_weights.pt")
