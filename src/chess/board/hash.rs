use super::Board;
use super::Piece;
use crate::chess::board::*;

impl Board {
    pub fn calculate_hash(&self) -> u64 {
        let mut super_hash = 0u64;
        self.piece_bb
            .into_iter()
            .enumerate()
            .for_each(|(piece_index, mut bb)| {
                while bb != EMPTY {
                    let square = bb.lsb().index();
                    super_hash ^= ZOBRIST_TABLE.pieces[piece_index][square];
                    bb.pop_lsb();
                }
            });

        super_hash ^= ZOBRIST_TABLE.castling[self.castling_rights as usize];
        if self.en_passant != EMPTY {
            super_hash ^= ZOBRIST_TABLE.en_passant[self.en_passant.lsb().file() as usize];
        }

        super_hash ^= ZOBRIST_TABLE.side_to_move * self.turn.index() as u64;

        super_hash
    }

    #[inline(always)]
    pub fn update_hash_piece<S: Side>(&mut self, p: Piece, square: Square) {
        self.hash ^= ZOBRIST_TABLE.pieces[p.index() + S::OFFSET][square.index()]
    }

    #[inline(always)]
    pub fn update_hash_caslte(&mut self, castling: u8) {
        self.hash ^= ZOBRIST_TABLE.castling[castling as usize];
    }

    #[inline(always)]
    pub fn update_move_hash(&mut self) {
        self.hash ^= ZOBRIST_TABLE.side_to_move;
    }

    #[inline(always)]
    pub fn update_en_passant_hash(&mut self) {
        if self.en_passant != EMPTY {
            self.hash ^= ZOBRIST_TABLE.en_passant[self.en_passant.lsb().file() as usize];
        }
    }
}

struct JenkinsRng {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

impl JenkinsRng {
    pub const fn new(seed: u64) -> Self {
        let mut rng = JenkinsRng {
            a: 0xf1ea_5eed,
            b: seed,
            c: seed,
            d: seed,
        };

        let mut i = 0;
        while i < 20 {
            rng.next();
            i += 1;
        }
        rng
    }

    const fn next(&mut self) -> u64 {
        let e = self.a.wrapping_sub(self.b.rotate_left(7));
        self.a = self.b ^ self.c.rotate_left(13);
        self.b = self.c.wrapping_add(self.d.rotate_left(37));
        self.c = self.d.wrapping_add(e);
        self.d = e.wrapping_add(self.a);
        self.d
    }
}

#[derive(Debug)]
pub struct ZobristTable {
    pub pieces: [[u64; 64]; 12],
    pub castling: [u64; 16],
    pub en_passant: [u64; 8],
    pub side_to_move: u64,
}

impl ZobristTable {
    const fn new() -> Self {
        let mut prng = JenkinsRng::new(12);

        let piece_nums: [[u64; 64]; 12] = {
            let mut table: [[u64; 64]; 12] = [[0u64; 64]; 12];
            let mut piece = 0;
            while piece < 12 {
                let mut square = 0;
                while square < 64 {
                    table[piece][square] = prng.next();
                    square += 1;
                }
                piece += 1
            }
            table
        };

        let castling_nums = {
            let mut num_castlings = 0;
            let mut table = [0u64; 16];
            while num_castlings < 16 {
                table[num_castlings] = prng.next();
                num_castlings += 1;
            }
            table
        };

        let en_passant_nums = {
            let mut num_en_passnats = 0;
            let mut table = [0u64; 8];
            while num_en_passnats < 8 {
                table[num_en_passnats] = prng.next();
                num_en_passnats += 1;
            }
            table
        };

        let side_to_move_num = prng.next();

        ZobristTable {
            pieces: piece_nums,
            castling: castling_nums,
            en_passant: en_passant_nums,
            side_to_move: side_to_move_num,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashList<const N: usize> {
    positions: [u64; N],
    count: usize,
}

impl<const N: usize> HashList<N> {
    pub fn new() -> Self {
        HashList {
            positions: [0u64; N],
            count: 0,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, hash: u64) {
        self.positions[self.count] = hash;
        self.count += 1;
    }

    #[inline(always)]
    pub fn pop(&mut self) {
        self.count -= 1;
    }

    #[inline(always)]
    pub fn half_move_iter(&self, num_halfmoves: u64) -> &[u64] {
        let num = self.count.saturating_sub(num_halfmoves as usize);
        &self.positions[num..self.count]
    }

    pub fn print(&self) {
        for m in self.half_move_iter(self.count as u64) {
            println!("{}", m)
        }
    }
}

pub const ZOBRIST_TABLE: ZobristTable = ZobristTable::new();
