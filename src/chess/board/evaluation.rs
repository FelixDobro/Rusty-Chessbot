use crate::chess::board::Board;
use crate::chess::board::bitboard::EMPTY as EMPTY_BB;
use crate::chess::constants::Piece::*;
use crate::chess::constants::{BlackSide, Piece, Side, WhiteSide};
use crate::chess::square::Square;

pub const CHECK_MATE: i16 = i16::MIN + 100;
// Values that cant be reached through normal evaluation / a bound for checking
pub const CHECK_MATE_THRESHOLD: i16 = 32_000;
pub const NEG_INFINITY: i16 = i16::MIN + 2;
pub const POSITIVE_INFINITY: i16 = -NEG_INFINITY;

impl Board {
    pub fn calc_phase(&self) -> i16 {
        let mut val = 0;

        val += 1.min(self.piece_bb[Queen.index() + WhiteSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Queen.index()];
        val += 2.min(self.piece_bb[Bishop.index() + WhiteSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Bishop.index()];
        val += 2.min(self.piece_bb[Rook.index() + WhiteSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Rook.index()];
        val += 2.min(self.piece_bb[Knight.index() + WhiteSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Knight.index()];

        val += 1.min(self.piece_bb[Queen.index() + BlackSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Queen.index()];
        val += 2.min(self.piece_bb[Bishop.index() + BlackSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Bishop.index()];
        val += 2.min(self.piece_bb[Rook.index() + BlackSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Rook.index()];
        val += 2.min(self.piece_bb[Knight.index() + BlackSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Knight.index()];

        val
    }

    #[inline(always)]
    pub fn calculate_mg(&self) -> i16 {
        let mut val = 0;

        for i in 0..6 {
            let mut piece_bb = self.piece_bb[WhiteSide::OFFSET + i];
            while piece_bb != EMPTY_BB {
                val += MG[WhiteSide::OFFSET + i][piece_bb.lsb().index()];
                val += MG_VALUE[i];
                piece_bb.pop_lsb();
            }
        }
        for i in 0..6 {
            let mut piece_bb = self.piece_bb[BlackSide::OFFSET + i];
            while piece_bb != EMPTY_BB {
                val -= MG[BlackSide::OFFSET + i][piece_bb.lsb().index()];
                val -= MG_VALUE[i];
                piece_bb.pop_lsb();
            }
        }

        val
    }

    #[inline(always)]
    pub fn calculate_eg(&self) -> i16 {
        let mut val = 0;

        for i in 0..6 {
            let mut piece_bb = self.piece_bb[WhiteSide::OFFSET + i];
            while piece_bb != EMPTY_BB {
                val += EG[WhiteSide::OFFSET + i][piece_bb.lsb().index()];
                val += EG_VALUE[i];
                piece_bb.pop_lsb();
            }
        }
        for i in 0..6 {
            let mut piece_bb = self.piece_bb[BlackSide::OFFSET + i];
            while piece_bb != EMPTY_BB {
                val -= EG[BlackSide::OFFSET + i][piece_bb.lsb().index()];
                val -= EG_VALUE[i];
                piece_bb.pop_lsb();
            }
        }

        val
    }

    pub fn eval(&self) -> i16 {
        let s_item = self.accum_stack.peek();
        let white = s_item[WhiteSide::INDEX];
        let black = s_item[BlackSide::INDEX];
        let out = if self.turn.is_white() {
            white.forawrd(&black)
        } else {
            black.forawrd(&white)
        } as f32;
        let cp = out / (256.0 * 64.0) * 400.0;
        cp.clamp(-CHECK_MATE_THRESHOLD as f32, CHECK_MATE_THRESHOLD as f32) as i16
    }

    pub fn piece_eval(&self, p: Piece) -> i16 {
        let game_phase_i32 = self.game_phase as i32;
        let mg_i32 = MG_VALUE[p.index()] as i32;
        let eg_i32 = EG_VALUE[p.index()] as i32;
        ((game_phase_i32 * mg_i32 + (24 - game_phase_i32) * eg_i32) / 24) as i16
    }

    #[inline(always)]
    pub fn add_eval<S: Side>(&mut self, p: Piece, s: Square) {
        self.eval_mg += MG[p.index() + S::OFFSET][s.index()] * S::MULTIPLIER;
        self.eval_eg += EG[p.index() + S::OFFSET][s.index()] * S::MULTIPLIER;
    }

    #[inline(always)]
    pub fn rm_eval<S: Side>(&mut self, p: Piece, s: Square) {
        self.eval_mg -= MG[p.index() + S::OFFSET][s.index()] * S::MULTIPLIER;
        self.eval_eg -= EG[p.index() + S::OFFSET][s.index()] * S::MULTIPLIER;
    }

    #[inline(always)]
    pub fn rm_p_eval<S: Side>(&mut self, p: Piece) {
        self.eval_mg -= MG_VALUE[p.index() + S::OFFSET];
        self.eval_eg -= EG_VALUE[p.index() + S::OFFSET];
    }

    #[inline(always)]
    pub fn add_p_eval<S: Side>(&mut self, p: Piece) {
        self.eval_mg += MG_VALUE[p.index() + S::OFFSET];
        self.eval_eg += EG_VALUE[p.index() + S::OFFSET];
    }

    #[inline(always)]
    pub fn result(&self, in_check: bool) -> i16 {
        return if in_check { CHECK_MATE } else { 0 };
    }
}

pub const PHASE_VALUES: [i16; 7] = [0, 1, 1, 2, 4, 0, 0];

#[inline(always)]
const fn flip(i: usize) -> usize {
    i ^ 56
}

pub static MG: [[i16; 64]; 12] = {
    let mut table = [[0i16; 64]; 12];

    table[Pawn.index() + BlackSide::OFFSET] = [
        0, 0, 0, 0, 0, 0, 0, 0, 98, 134, 61, 95, 68, 126, 34, -11, -6, 7, 26, 31, 65, 56, 25, -20,
        -14, 13, 6, 21, 23, 12, 17, -23, -27, -2, -5, 12, 17, 6, 10, -25, -26, -4, -4, -10, 3, 3,
        33, -12, -35, -1, -20, -23, -15, 24, 38, -22, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    table[Knight.index() + BlackSide::OFFSET] = [
        -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37, 65,
        84, 129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8, -23, -9,
        12, 10, 19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58, -33, -17,
        -28, -19, -23,
    ];

    table[Bishop.index() + BlackSide::OFFSET] = [
        -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40, 35,
        50, 37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15, 15, 14,
        27, 18, 10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
    ];

    table[Rook.index() + BlackSide::OFFSET] = [
        32, 42, 32, 51, 63, 9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45, 61,
        16, -24, -11, 7, 26, 24, 35, -8, -20, -36, -26, -12, -1, 9, -7, 6, -23, -45, -25, -16, -17,
        3, 0, -5, -33, -44, -16, -20, -9, -1, 11, -6, -71, -19, -13, 1, 17, 16, 7, -37, -26,
    ];

    table[Queen.index() + BlackSide::OFFSET] = [
        -28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29, 56,
        47, 57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2, -11,
        -2, -5, 2, 14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31, -50,
    ];

    table[King.index() + BlackSide::OFFSET] = [
        -65, 23, 16, -15, -56, -34, 2, 13, 29, -1, -20, -7, -8, -4, -38, -29, -9, 24, 2, -16, -20,
        6, 22, -22, -17, -20, -12, -27, -30, -25, -14, -36, -49, -1, -27, -39, -46, -44, -33, -51,
        -14, -14, -22, -46, -44, -30, -15, -27, 1, 7, -8, -64, -43, -16, 9, 8, -15, 36, 12, -54, 8,
        -28, 24, 14,
    ];
    let mut piece = 0;
    while piece < 6 {
        let mut sq = 0;
        while sq < 64 {
            table[WhiteSide::OFFSET + piece][sq] = table[BlackSide::OFFSET + piece][flip(sq)];
            sq += 1;
        }
        piece += 1
    }
    table
};

pub static EG: [[i16; 64]; 12] = {
    let mut table = [[0i16; 64]; 12];

    table[Pawn.index() + BlackSide::OFFSET] = [
        0, 0, 0, 0, 0, 0, 0, 0, 178, 173, 158, 134, 147, 132, 165, 187, 94, 100, 85, 67, 56, 53,
        82, 84, 32, 24, 13, 5, -2, 4, 17, 17, 13, 9, -3, -7, -7, -8, 3, -1, 4, 7, -6, 1, 0, -5, -1,
        -8, 13, 8, 8, 10, 13, 0, 2, -7, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    table[Knight.index() + BlackSide::OFFSET] = [
        -58, -38, -13, -28, -31, -27, -63, -99, -25, -8, -25, -2, -9, -25, -24, -52, -24, -20, 10,
        9, -1, -9, -19, -41, -17, 3, 22, 22, 22, 11, 8, -18, -18, -6, 16, 25, 16, 17, 4, -18, -23,
        -3, -1, 15, 10, -3, -20, -22, -42, -20, -10, -5, -2, -20, -23, -44, -29, -51, -23, -15,
        -22, -18, -50, -64,
    ];

    table[Bishop.index() + BlackSide::OFFSET] = [
        -14, -21, -11, -8, -7, -9, -17, -24, -8, -4, 7, -12, -3, -13, -4, -14, 2, -8, 0, -1, -2, 6,
        0, 4, -3, 9, 12, 9, 14, 10, 3, 2, -6, 3, 13, 19, 7, 10, -3, -9, -12, -3, 8, 10, 13, 3, -7,
        -15, -14, -18, -7, -1, 4, -9, -15, -27, -23, -9, -23, -5, -9, -16, -5, -17,
    ];

    table[Rook.index() + BlackSide::OFFSET] = [
        13, 10, 18, 15, 12, 12, 8, 5, 11, 13, 13, 11, -3, 3, 8, 3, 7, 7, 7, 5, 4, -3, -5, -3, 4, 3,
        13, 1, 2, 1, -1, 2, 3, 5, 8, 4, -5, -6, -8, -11, -4, 0, -5, -1, -7, -12, -8, -16, -6, -6,
        0, 2, -9, -9, -11, -3, -9, 2, 3, -1, -5, -13, 4, -20,
    ];

    table[Queen.index() + BlackSide::OFFSET] = [
        -9, 22, 22, 27, 27, 19, 10, 20, -17, 20, 32, 41, 58, 25, 30, 0, -20, 6, 9, 49, 47, 35, 19,
        9, 3, 22, 24, 45, 57, 40, 57, 36, -18, 28, 19, 47, 31, 34, 39, 23, -16, -27, 15, 6, 9, 17,
        10, 5, -22, -23, -30, -16, -16, -23, -36, -32, -33, -28, -22, -43, -5, -32, -20, -41,
    ];

    table[King.index() + BlackSide::OFFSET] = [
        -74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15, 20,
        45, 44, 13, -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19, -3, 11,
        21, 23, 16, 7, -9, -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28, -14, -24, -43,
    ];

    let mut piece = 0;
    while piece < 6 {
        let mut sq = 0;
        while sq < 64 {
            table[WhiteSide::OFFSET + piece][sq] = table[BlackSide::OFFSET + piece][flip(sq)];
            sq += 1;
        }
        piece += 1
    }

    table
};

pub const MG_VALUE: [i16; 12] = [82, 337, 365, 477, 1025, 0, -82, -337, -365, -477, -1025, -0];
pub const EG_VALUE: [i16; 12] = [94, 281, 297, 512, 936, 0, -94, -281, -297, -512, -936, -0];
