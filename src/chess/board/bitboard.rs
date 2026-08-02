use super::Square;

use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
#[repr(transparent)]
pub struct Bitboard(u64);

impl Bitboard {
    #[inline(always)]
    pub const fn from_u64(board: u64) -> Self {
        Bitboard(board)
    }

    #[inline(always)]
    pub const fn with_one_bit(square: Square) -> Self {
        Bitboard(1u64 << square.u8())
    }

    #[inline(always)]
    pub fn from_squares(squares: Vec<Square>) -> Self {
        let mut board = EMPTY;
        squares
            .iter()
            .for_each(|&sq| board |= Self::with_one_bit(sq));
        board
    }

    #[inline(always)]
    pub const fn u64(self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub const fn count_ones(self) -> u32 {
        self.0.count_ones()
    }

    #[inline(always)]
    pub fn lsb(self) -> Square {
        Square::from_u8(self.0.trailing_zeros() as u8)
    }

    #[inline(always)]
    pub fn pop_lsb(&mut self) {
        self.0 &= self.0 - 1;
    }

    pub fn print(self) -> () {
        println!("  A B C D E F G H");

        for rank in (0..8).rev() {
            print!("{} ", rank + 1);

            for file in 0..8 {
                let relevant = self >> (rank * 8 + file);
                let bit = relevant & Self::from_u64(1);
                print!("{} ", bit.0);
            }
            println!();
        }
        println!("  A B C D E F G H");
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitboard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Shl<u8> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn shl(self, rhs: u8) -> Self::Output {
        Bitboard(self.0 << rhs)
    }
}

impl ShlAssign<u8> for Bitboard {
    #[inline(always)]
    fn shl_assign(&mut self, rhs: u8) {
        self.0 <<= rhs;
    }
}

impl Shr<u8> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn shr(self, rhs: u8) -> Self::Output {
        Bitboard(self.0 >> rhs)
    }
}

impl ShrAssign<u8> for Bitboard {
    #[inline(always)]
    fn shr_assign(&mut self, rhs: u8) {
        self.0 >>= rhs;
    }
}

impl Not for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}

pub const PAWN_W_DEFAULT: Bitboard = Bitboard(0x000000000000FF00);
pub const BISHOP_W_DEFAULT: Bitboard = Bitboard(0x0000000000000024);
pub const KNIGHT_W_DEFAULT: Bitboard = Bitboard(0x0000000000000042);
pub const ROOK_W_DEFAULT: Bitboard = Bitboard(0x0000000000000081);
pub const QUEEN_W_DEFAULT: Bitboard = Bitboard(0x0000000000000008);
pub const KING_W_DEFAULT: Bitboard = Bitboard(0x0000000000000010);

pub const PAWN_B_DEFAULT: Bitboard = Bitboard(0x00FF000000000000);
pub const BISHOP_B_DEFAULT: Bitboard = Bitboard(0x2400000000000000);
pub const KNIGHT_B_DEFAULT: Bitboard = Bitboard(0x4200000000000000);
pub const ROOK_B_DEFAULT: Bitboard = Bitboard(0x8100000000000000);
pub const QUEEN_B_DEFAULT: Bitboard = Bitboard(0x0800000000000000);
pub const KING_B_DEFAULT: Bitboard = Bitboard(0x1000000000000000);

pub const EMPTY: Bitboard = Bitboard(0x0000000000000000);
pub const FULL: Bitboard = Bitboard(0xFFFFFFFFFFFFFFFF);

pub const RANK_1: Bitboard = Bitboard(0x00000000000000FF);
pub const RANK_2: Bitboard = Bitboard(0x000000000000FF00);
pub const RANK_3: Bitboard = Bitboard(0x0000000000FF0000);
pub const RANK_4: Bitboard = Bitboard(0x00000000FF000000);
pub const RANK_5: Bitboard = Bitboard(0x000000FF00000000);
pub const RANK_6: Bitboard = Bitboard(0x0000FF0000000000);
pub const RANK_7: Bitboard = Bitboard(0x00FF000000000000);
pub const RANK_8: Bitboard = Bitboard(0xFF00000000000000);

pub const FILE_A: Bitboard = Bitboard(0x0101010101010101);
pub const FILE_B: Bitboard = Bitboard(0x0202020202020202);
pub const FILE_C: Bitboard = Bitboard(0x0404040404040404);
pub const FILE_D: Bitboard = Bitboard(0x0808080808080808);
pub const FILE_E: Bitboard = Bitboard(0x1010101010101010);
pub const FILE_F: Bitboard = Bitboard(0x2020202020202020);
pub const FILE_G: Bitboard = Bitboard(0x4040404040404040);
pub const FILE_H: Bitboard = Bitboard(0x8080808080808080);

pub const WHITE_KING_CASTLE_BLOCKERS: Bitboard =
    Bitboard((1u64 << Square::F1.u8()) | (1u64 << Square::G1.u8()));
pub const BLACK_KING_CASTLE_BLOCKERS: Bitboard =
    Bitboard((1u64 << Square::F8.u8()) | (1u64 << Square::G8.u8()));
pub const WHITE_QUEEN_CASTLE_BLOCKERS: Bitboard =
    Bitboard((1u64 << Square::B1.u8()) | (1u64 << Square::C1.u8()) | (1u64 << Square::D1.u8()));
pub const BLACK_QUEEN_CASTLE_BLOCKERS: Bitboard =
    Bitboard((1u64 << Square::B8.u8()) | (1u64 << Square::C8.u8()) | (1u64 << Square::D8.u8()));

// Korrigierte Defaults (In deinem Prompt war Black/Occupied identisch zu White)
pub const DEFAULT_COLOR_W: Bitboard = Bitboard(0x000000000000FFFF);
pub const DEFAULT_COLOR_B: Bitboard = Bitboard(0xFFFF000000000000);
pub const DEFAULT_OCCUPIED: Bitboard = Bitboard(0xFFFF00000000FFFF);

// when performing a double move this table maps m.from sqaures to en_passant bitboards
pub static EN_PESSANT_UPDATES: [Bitboard; 64] = {
    let mut table = [EMPTY; 64];
    let mut square = 0;
    while square < 64 {
        let board = 1u64 << square;
        if (board & RANK_2.u64()) != 0 {
            table[square as usize] = Bitboard::with_one_bit(Square::from_u8(square + 8));
        } else if (board & RANK_7.u64()) != 0 {
            table[square as usize] = Bitboard::with_one_bit(Square::from_u8(square - 8));
        }
        square += 1;
    }

    table
};

// maps move.to squares of an en passant moves to bitboards of captured pawns
pub static EN_PASSANT_RM_SQUARES: [Bitboard; 64] = {
    let mut table = [EMPTY; 64];

    let mut square = 0;
    while square < 64 {
        let board = 1u64 << square;
        if (board & RANK_6.u64()) != 0 {
            table[square as usize] = Bitboard::with_one_bit(Square::from_u8(square - 8));
        } else if (board & RANK_3.u64()) != 0 {
            table[square as usize] = Bitboard::with_one_bit(Square::from_u8(square + 8));
        }
        square += 1;
    }

    table
};

pub static STRAIGHT_LINES: [Bitboard; 64] = {
    let mut table = [EMPTY; 64];
    let mut sq = 0;

    while sq < 64 {
        let board = 1u64 << sq;
        let mut vertical_board = board;

        let mut horizontal_board = board;
        let mut slider = 0;
        while slider < 8 {
            vertical_board |=
                ((vertical_board << 8) | (vertical_board >> 8)) & !(RANK_1.u64() | RANK_8.u64());
            horizontal_board |= ((horizontal_board << 1) | (horizontal_board >> 1))
                & !(FILE_A.u64() | FILE_H.u64());
            slider += 1;
        }
        table[sq as usize] = Bitboard::from_u64((horizontal_board | vertical_board) & !board);
        sq += 1;
    }
    table
};

pub static DIAGONAL_LINES: [Bitboard; 64] = {
    let mut table = [EMPTY; 64];
    let mut sq = 0;

    while sq < 64 {
        let board = 1u64 << sq;
        let mut left_prefix = board;

        let mut right_prefix = board;
        let mut slider: u32 = 0;
        let mut moves = 0u64;
        while slider < 6 {
            slider += 1;
            left_prefix = (left_prefix >> 1) & !(FILE_A.u64() | FILE_H.u64());
            right_prefix = (right_prefix << 1) & !(FILE_A.u64() | FILE_H.u64());

            moves |= ((left_prefix >> slider * 8) & !RANK_1.u64())
                | ((left_prefix << 8 * slider) & !(RANK_8.u64()));
            moves |= ((right_prefix >> slider * 8) & !RANK_1.u64())
                | ((right_prefix << 8 * slider) & !(RANK_8.u64()));
        }
        table[sq as usize] = Bitboard::from_u64(moves);
        sq += 1;
    }

    table
};

// Needed for const PDEP Intrinsic functionallity. Slow but does not matter since precomputed

const fn const_pdep(index: u64, mut move_rays: u64) -> u64 {
    let mut start = 0;
    let mut result = 0;

    while move_rays != 0 {
        // LSB
        let bit = move_rays & move_rays.wrapping_neg();

        if (index >> start) & 1 != 0 {
            result |= bit;
        }

        move_rays ^= bit;
        start += 1;
    }
    result
}

pub static DIAG_LINES_MAGIC: [[Bitboard; 512]; 64] = {
    let mut table: [[Bitboard; 512]; 64] = [[EMPTY; 512]; 64];

    let mut sq = 0;

    while sq < 64 {
        let move_mask = DIAGONAL_LINES[sq as usize].u64();

        let mut unique_index = 0;

        while unique_index < (1u64 << move_mask.count_ones()) {
            let occupancy = const_pdep(unique_index, move_mask);

            let mut tmp_square: i32 = sq as i32;
            let mut moves = EMPTY.u64();

            // right up
            while tmp_square % 8 < 7 && tmp_square < 56 {
                tmp_square += 9;
                let bit = 1u64 << tmp_square;
                moves |= bit;
                if occupancy & bit != 0 {
                    break;
                }
            }

            let mut tmp_square: i32 = sq as i32;
            // left up
            while tmp_square % 8 > 0 && tmp_square < 56 {
                tmp_square += 7;
                let bit = 1u64 << tmp_square;
                moves |= bit;
                if occupancy & bit != 0 {
                    break;
                }
            }

            let mut tmp_square: i32 = sq as i32;
            // right down
            while tmp_square % 8 < 7 && tmp_square > 7 {
                tmp_square -= 7;
                let bit = 1u64 << tmp_square;
                moves |= bit;
                if occupancy & bit != 0 {
                    break;
                }
            }

            let mut tmp_square = sq as i32;
            // left down
            while tmp_square % 8 > 0 && tmp_square > 7 {
                tmp_square -= 9;
                let bit = 1u64 << tmp_square;
                moves |= bit;
                if occupancy & bit != 0 {
                    break;
                }
            }

            table[sq][unique_index as usize] = Bitboard::from_u64(moves);
            unique_index += 1;
        }
        sq += 1;
    }

    table
};

#[allow(long_running_const_eval)]
pub static STRAIGHT_LINES_MAGIC: [[Bitboard; 4096]; 64] = {
    let mut table = [[EMPTY; 4096]; 64];

    let mut sq: u32 = 0;

    while sq < 64 {
        let move_mask = STRAIGHT_LINES[sq as usize].u64();

        let mut unique_index = 0;

        while unique_index < (1u64 << move_mask.count_ones()) {
            let occupancy = const_pdep(unique_index as u64, move_mask);

            let mut tmp_square: i32 = sq as i32;
            let mut moves = EMPTY.u64();

            while tmp_square % 8 > 0 {
                tmp_square -= 1;
                let bit = 1u64 << tmp_square;
                moves |= bit;
                if occupancy & bit != 0 {
                    break;
                }
            }

            let mut tmp_square = sq;
            while tmp_square % 8 < 7 {
                tmp_square += 1;
                let bit = 1u64 << tmp_square;
                moves |= bit;
                if occupancy & bit != 0 {
                    break;
                }
            }

            let mut tmp_square = sq;
            while tmp_square < 56 {
                tmp_square += 8;
                let bit = 1u64 << tmp_square;
                moves |= bit;
                if occupancy & bit != 0 {
                    break;
                }
            }

            let mut tmp_square = sq;
            while tmp_square > 7 {
                tmp_square -= 8;
                let bit = 1u64 << tmp_square;
                moves |= bit;
                if occupancy & bit != 0 {
                    break;
                }
            }

            table[sq as usize][unique_index as usize] = Bitboard::from_u64(moves);

            unique_index += 1;
        }
        sq += 1;
    }

    table
};

pub static PAWN_ATTACKS: [[Bitboard; 64]; 2] = {
    let mut map: [[Bitboard; 64]; 2] = [[EMPTY; 64]; 2];
    let mut sq: u8 = 0;
    while sq < 64 {
        let board = 1u64 << sq;

        // left right
        let prefix = ((board << 1) & !FILE_A.u64()) | ((board >> 1) & !FILE_H.u64());

        let white_attacks = prefix << 8;
        let black_attacks = prefix >> 8;
        map[0][sq as usize] = Bitboard::from_u64(white_attacks);
        map[1][sq as usize] = Bitboard::from_u64(black_attacks);
        sq += 1;
    }
    map
};

pub static KING_PATTERNS: [Bitboard; 64] = {
    let mut map: [Bitboard; 64] = [EMPTY; 64];
    let mut square = 0;
    while square < 64 {
        map[square as usize] = pre_calculate_king_moves(square);
        square += 1;
    }
    map
};

const fn pre_calculate_king_moves(sq: u8) -> Bitboard {
    let board = 1u64 << sq;

    // left right
    let prefix = ((board << 1) & !FILE_A.u64()) | ((board >> 1) & !FILE_H.u64());
    // prefix + up down
    let first_half = prefix | (prefix << 8) | (prefix >> 8);

    Bitboard::from_u64(first_half | (board >> 8) | (board << 8))
}

pub static KNIGHT_PATTERNS: [Bitboard; 64] = {
    let mut table = [EMPTY; 64];
    let mut square = 0;
    while square < 64 {
        table[square as usize] = pre_calculate_knight_moves(square);
        square += 1;
    }
    table
};

const fn pre_calculate_knight_moves(sq: u8) -> Bitboard {
    let board = 1u64 << sq;

    // 2 horizontal 1 vertical
    let mut prefix = (board << 16) | (board >> 16);
    let first_half = ((prefix << 1) & !FILE_A.u64()) | ((prefix >> 1) & !FILE_H.u64());

    // 1 horizontal 2 vertical
    prefix = (board << 8) | (board >> 8);

    let un_bit_board = ((prefix >> 2) & !(FILE_G.u64() | FILE_H.u64()))
        | ((prefix << 2) & !(FILE_A.u64() | FILE_B.u64()))
        | (first_half);
    Bitboard::from_u64(un_bit_board)
}
