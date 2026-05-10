#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};
use std::sync::LazyLock;

pub const NUM_PIECES: usize = 6;
pub const NUM_COLORS: usize = 2;

pub const SYMBOLS: [char; 12] = ['♟', '♝', '♞', '♜', '♛', '♚', '♙', '♗', '♘', '♖', '♕', '♔'];

pub const EMPTY: u64 = 0x0000000000000000;
pub const FULL: u64 = 0xFFFFFFFFFFFFFFFF;

pub const RANK_1: u64 = 0x00000000000000FF;
pub const RANK_2: u64 = 0x000000000000FF00;
pub const RANK_3: u64 = 0x0000000000FF0000;
pub const RANK_4: u64 = 0x00000000FF000000;
pub const RANK_5: u64 = 0x000000FF00000000;
pub const RANK_6: u64 = 0x0000FF0000000000;
pub const RANK_7: u64 = 0x00FF000000000000;
pub const RANK_8: u64 = 0xFF00000000000000;

pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = 0x0202020202020202;
pub const FILE_C: u64 = 0x0404040404040404;
pub const FILE_D: u64 = 0x0808080808080808;
pub const FILE_E: u64 = 0x1010101010101010;
pub const FILE_F: u64 = 0x2020202020202020;
pub const FILE_G: u64 = 0x4040404040404040;
pub const FILE_H: u64 = 0x8080808080808080;

// when performing a double move this table maps m.from sqaures to en_passant bitboards
pub const EN_PESSANT_UPDATES: [u64; 64] = {
    let mut table: [u64; 64] = [0u64; 64];

    let mut square = 0;
    while square < 64 {
        let board = 1u64 << square;
        if (board & RANK_2) != 0  {
            table[square as usize] = 1u64 << (square + 8);
        }
        else if (board & RANK_7) != 0 {
            table[square as usize] = 1u64 << (square - 8);
        }
        square += 1;
    }

    table
};

// maps move.to squares of an en passant moves to bitboards of captured pawns
pub const EN_PASSANT_RM_SQUARES: [u64; 64] = {
    let mut table: [u64; 64] = [0u64; 64];

    let mut square = 0;
    while square < 64 {
        let board = 1u64 << square;
        if (board & RANK_6) != 0  {
            table[square as usize] = 1u64 << (square - 8);
        }
        else if (board & RANK_3) != 0 {
            table[square as usize] = 1u64 << (square + 8);
        }
        square += 1;
    }

    table
};


#[repr(u8)]
pub enum CASTLING_RIGHTS {
    KingCastleWhite = 1,
    KingCastleBlack = 2,
    QueenCastleWhite = 4,
    QueenCastleBlack = 8
}

impl CASTLING_RIGHTS {

    #[inline(always)]
    pub const fn index(self) -> u8 {
        self as u8
    } 
}

pub const CASTLING_RIGHTS: [u8; 64] = {
    let mut i = 0;
    let mut table: [u8; 64] = [0u8; 64];
    while i < 64 {
        table[i as usize] = match Square::from_u8(i) {
            Square::A1 => CASTLING_RIGHTS::QueenCastleWhite.index(),
            Square::A8 => CASTLING_RIGHTS::QueenCastleBlack.index(),
            Square::H1 => CASTLING_RIGHTS::KingCastleWhite.index(),
            Square::H8 => CASTLING_RIGHTS::KingCastleBlack.index(),
            _ => 0

        };
        i += 1;
    }
    table
};

pub struct CastlingMechanics {
    pub rook_disappears: Square,
    pub rook_appears: Square,
    pub king_disappears: Square,
    pub king_appears: Square,
    pub rook_movement: u64,
    pub king_movement: u64,
    pub combined_movement: u64,
    pub castling_rights_update: u8
}

impl CastlingMechanics {
    
    pub const fn new(rook_from: Square, rook_to: Square, king_from: Square, king_to: Square, update: u8) -> Self {
        let from_board_rook = 1u64 << rook_from.index();
        let to_board_rook = 1u64 << rook_to.index();
        let from_board_king = 1u64 << king_from.index();
        let to_board_king = 1u64 << king_to.index();

        let rook_movement = from_board_rook ^ to_board_rook;
        let king_movement = from_board_king ^ to_board_king;

        let combined_movement = rook_movement | king_movement;

        Self { 
            rook_disappears: rook_from,
            rook_appears: rook_to,
            king_disappears: king_from,
            king_appears: king_to,
            rook_movement: rook_movement,
            king_movement: king_movement,
            combined_movement: combined_movement,
            castling_rights_update: update
            }
    }
}

pub const CASTLING_TABLE: [[CastlingMechanics; 2]; 2] = 
[
    [
        CastlingMechanics::new(
            Square::H1,
            Square::F1,
            Square::E1,
            Square::G1,
            CASTLING_RIGHTS::KingCastleWhite.index()
        ),
        CastlingMechanics::new(
            Square::A1,
            Square::D1,
            Square::E1,
            Square::C1,
            CASTLING_RIGHTS::QueenCastleWhite.index()
        )
    ],
    [
        CastlingMechanics::new(
            Square::H8,
            Square::F8,
            Square::E8,
            Square::G8,
            CASTLING_RIGHTS::KingCastleBlack.index()
        ),
        CastlingMechanics::new(
            Square::A8,
            Square::D8,
            Square::E8,
            Square::C8,
            CASTLING_RIGHTS::QueenCastleBlack.index()
        )
    ]
];


pub const STRAIGHT_LINES: [u64; 64] = {
    let mut table: [u64; 64] = [0u64; 64];
    let mut sq = 0;

    while sq < 64 {
        let board = 1u64 << sq;
        let mut vertical_board = board;

        let mut horizontal_board = board;
        let mut slider = 0;
        while slider < 8 {
            vertical_board |= ((vertical_board << 8) | (vertical_board >> 8)) & !(RANK_1 | RANK_8);
            horizontal_board |=
                ((horizontal_board << 1) | (horizontal_board >> 1)) & !(FILE_A | FILE_H);
            slider += 1;
        }
        table[sq as usize] = (horizontal_board | vertical_board) & !board;
        sq += 1;
    }

    table
};

pub const DIAGONAL_LINES: [u64; 64] = {
    let mut table: [u64; 64] = [0u64; 64];
    let mut sq = 0;

    while sq < 64 {
        let board = 1u64 << sq;
        let mut left_prefix = board;

        let mut right_prefix = board;
        let mut slider: u32 = 0;
        let mut moves = 0u64;
        while slider < 6 {
            slider += 1;
            left_prefix = (left_prefix >> 1) & !(FILE_A | FILE_H);
            right_prefix = (right_prefix << 1) & !(FILE_A | FILE_H);

            moves |=
                ((left_prefix >> slider * 8) & !RANK_1) | ((left_prefix << 8 * slider) & !(RANK_8));
            moves |= ((right_prefix >> slider * 8) & !RANK_1)
                | ((right_prefix << 8 * slider) & !(RANK_8));
        }
        table[sq as usize] = moves;
        sq += 1;
    }

    table
};

// Needed for const PDEP Intrinsic functionallity. Slow but does not matter since precomputed
const fn const_PDEP(index: u64, mut move_rays: u64) -> u64 {
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

pub static DIAG_LINES_MAGIC: LazyLock<Box<[[u64; 512]; 64]>> = LazyLock::new(|| {
    let mut table: Box<[[u64; 512]; 64]> = Box::new([[0u64; 512]; 64]);

    let mut sq = 0;

    while sq < 64 {
        let move_mask = DIAGONAL_LINES[sq as usize];

        let mut unique_index = 0;

        while unique_index < (1u64 << move_mask.count_ones()) {
            let occupancy = const_PDEP(unique_index, move_mask);

            let mut tmp_square: i32 = sq as i32;
            let mut moves = EMPTY;

            // right up
            while tmp_square % 8 < 7 && tmp_square < 56{
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

            table[sq][unique_index as usize] = moves;
            unique_index += 1;
        }
        sq += 1;
    }

    table
});

pub static STRAIGHT_LINES_MAGIC: LazyLock<Box<[[u64; 4096]; 64]>> = LazyLock::new(|| {
    let mut table: Box<[[u64; 4096]; 64]> = Box::new([[0u64; 4096]; 64]);

    let mut sq: u32 = 0;

    while sq < 64 {
        let move_mask = STRAIGHT_LINES[sq as usize];

        let mut unique_index = 0;

        while unique_index < (1u64 << move_mask.count_ones()) {
            let occupancy = const_PDEP(unique_index as u64, move_mask);

            let mut tmp_square: i32 = sq as i32;
            let mut moves = EMPTY;

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

            table[sq as usize][unique_index as usize] = moves;

            unique_index += 1;
        }
        sq += 1;
    }

    table
});

pub const PAWN_ATTACKS: [[u64; 64]; 2] = {
    let mut map: [[u64; 64]; 2] = [[0u64; 64]; 2];
    let mut sq: u8 = 0;
    while sq < 64 {
        let board = 1u64 << sq;

        // left right
        let prefix = ((board << 1) & !FILE_A) | ((board >> 1) & !FILE_H);

        let white_attacks = prefix << 8;
        let black_attacks = prefix >> 8;
        map[0][sq as usize] = white_attacks;
        map[1][sq as usize] = black_attacks;
        sq += 1;
    }
    map
};

pub const KING_PATTERNS: [u64; 64] = {
    let mut map: [u64; 64] = [0u64; 64];
    let mut square = 0;
    while square < 64 {
        map[square as usize] = pre_calculate_king_moves(square);
        square += 1;
    }
    map
};

const fn pre_calculate_king_moves(sq: u8) -> u64 {
    let board = 1u64 << sq;

    // left right
    let prefix = ((board << 1) & !FILE_A) | ((board >> 1) & !FILE_H);
    // prefix + up down
    let first_half = prefix | (prefix << 8) | (prefix >> 8);

    first_half | (board >> 8) | (board << 8)
}

pub const KNIGHT_PATTERNS: [u64; 64] = {
    let mut table = [0u64; 64];
    let mut square = 0;
    while square < 64 {
        table[square as usize] = pre_calculate_knight_moves(square);
        square += 1;
    }
    table
};

const fn pre_calculate_knight_moves(sq: u8) -> u64 {
    let board = 1u64 << sq;

    // 2 horizontal 1 vertical
    let mut prefix = (board << 16) | (board >> 16);
    let first_half = ((prefix << 1) & !FILE_A) | ((prefix >> 1) & !FILE_H);

    // 1 horizontal 2 vertical
    prefix = (board << 8) | (board >> 8);

    ((prefix >> 2) & !(FILE_G | FILE_H)) | ((prefix << 2) & !(FILE_A | FILE_B)) | (first_half)
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Piece {
    Pawn = 0,
    Bishop = 1,
    Knight = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
    Empty = 6,
}

impl Piece {
    #[inline]
    pub fn index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn from_u8(value: u8) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Square {
    A1 = 0,
    B1 = 1,
    C1 = 2,
    D1 = 3,
    E1 = 4,
    F1 = 5,
    G1 = 6,
    H1 = 7,
    A2 = 8,
    B2 = 9,
    C2 = 10,
    D2 = 11,
    E2 = 12,
    F2 = 13,
    G2 = 14,
    H2 = 15,
    A3 = 16,
    B3 = 17,
    C3 = 18,
    D3 = 19,
    E3 = 20,
    F3 = 21,
    G3 = 22,
    H3 = 23,
    A4 = 24,
    B4 = 25,
    C4 = 26,
    D4 = 27,
    E4 = 28,
    F4 = 29,
    G4 = 30,
    H4 = 31,
    A5 = 32,
    B5 = 33,
    C5 = 34,
    D5 = 35,
    E5 = 36,
    F5 = 37,
    G5 = 38,
    H5 = 39,
    A6 = 40,
    B6 = 41,
    C6 = 42,
    D6 = 43,
    E6 = 44,
    F6 = 45,
    G6 = 46,
    H6 = 47,
    A7 = 48,
    B7 = 49,
    C7 = 50,
    D7 = 51,
    E7 = 52,
    F7 = 53,
    G7 = 54,
    H7 = 55,
    A8 = 56,
    B8 = 57,
    C8 = 58,
    D8 = 59,
    E8 = 60,
    F8 = 61,
    G8 = 62,
    H8 = 63,
}

impl Square {
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn from_u8(value: u8) -> Self {
        
        unsafe { std::mem::transmute(value) }
    }
}



#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
    Empty = 2,
}

impl Color {
    #[inline(always)]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub const fn offset(self) -> usize {
        self as usize * NUM_PIECES
    }
    
    #[inline(always)]
    pub const fn opposite(self) -> Color {
        
        unsafe { std::mem::transmute((self as usize ^ 1) as u8) }
    }
}


pub trait Side {

    const INDEX: usize;
    const OFFSET: usize;
    const UP: i8;
    const LAST_RANK: u64;
    const DOUBLE_PUSH_RANK: u64;
    type OPPOSITE: Side;

    #[inline(always)]
    fn shift_up(bb: u64) -> u64;

    #[inline(always)]
    fn pawn_attack_pattern_l(bb: u64) -> u64;

    #[inline(always)]
    fn pawn_attack_pattern_r(bb: u64) -> u64;

}

pub struct WhiteSide;
impl Side for WhiteSide {
    const INDEX: usize = 0;
    const OFFSET: usize = 0;
    const UP: i8 = 8;
    const LAST_RANK: u64 = RANK_8;
    const DOUBLE_PUSH_RANK: u64 = RANK_4;
    
    
    type OPPOSITE = BlackSide;

    #[inline(always)]
    fn shift_up(bb: u64) -> u64 {
        bb << 8
    }

    #[inline(always)]
    fn pawn_attack_pattern_l(bb: u64) -> u64 {
        (bb << 7) & !FILE_H
    }

    #[inline(always)]
    fn pawn_attack_pattern_r(bb: u64) -> u64 {
        (bb << 9) & !FILE_A
    }
    


}

pub struct BlackSide;
impl Side for BlackSide {
    const INDEX: usize = 1;
    const OFFSET: usize = NUM_PIECES as usize;
    const UP: i8 = -8;
    const LAST_RANK: u64 = RANK_1;
    const DOUBLE_PUSH_RANK: u64 = RANK_5;
    type OPPOSITE = WhiteSide;

    #[inline(always)]
    fn shift_up(bb: u64) -> u64 {
        bb >> 8
    }
    
    #[inline(always)]
    fn pawn_attack_pattern_l(bb: u64) -> u64 {
        (bb >> 9)  & !FILE_A
    }

    #[inline(always)]
    fn pawn_attack_pattern_r(bb: u64) -> u64 {
        
        (bb >> 7) & !FILE_H
    }
}


pub fn print_bitboard(board: u64) -> () {
    println!("  A B C D E F G H");

    for rank in (0..8).rev() {
        print!("{} ", rank + 1);

        for file in 0..8 {
            let relevant = board >> (rank * 8 + file);
            let bit = relevant & 1;
            print!("{} ", bit);
        }
        println!();
    }
    println!("  A B C D E F G H");
}
