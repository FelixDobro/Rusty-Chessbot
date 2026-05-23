#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};

use rayon::iter::Empty;

use crate::chess::square::Square;

use crate::chess::board::bitboard::{*};

pub const NUM_PIECES: usize = 6;
pub const NUM_COLORS: usize = 2;

pub const SYMBOLS: [char; 12] = ['♟', '♝', '♞', '♜', '♛', '♚', '♙', '♗', '♘', '♖', '♕', '♔'];



#[repr(u8)]
pub enum CastlingRights {
    KingCastleWhite = 1,
    KingCastleBlack = 2,
    QueenCastleWhite = 4,
    QueenCastleBlack = 8
}

impl CastlingRights {

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
            Square::A1 => CastlingRights::QueenCastleWhite.index(),
            Square::A8 => CastlingRights::QueenCastleBlack.index(),
            Square::H1 => CastlingRights::KingCastleWhite.index(),
            Square::H8 => CastlingRights::KingCastleBlack.index(),
            Square::E1 => CastlingRights::QueenCastleWhite.index() + CastlingRights::KingCastleWhite.index(),
            Square::E8 => CastlingRights::QueenCastleBlack.index() + CastlingRights::KingCastleBlack.index(),
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
    pub rook_movement: Bitboard,
    pub king_movement: Bitboard,
    pub combined_movement: Bitboard,
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
            rook_movement: Bitboard::from_u64(rook_movement),
            king_movement: Bitboard::from_u64(king_movement),
            combined_movement: Bitboard::from_u64(combined_movement),
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
            CastlingRights::QueenCastleWhite.index() + CastlingRights::KingCastleWhite.index()
        ),
        CastlingMechanics::new(
            Square::A1,
            Square::D1,
            Square::E1,
            Square::C1,
            CastlingRights::QueenCastleWhite.index() + CastlingRights::KingCastleWhite.index()
        )
    ],
    [
        CastlingMechanics::new(
            Square::H8,
            Square::F8,
            Square::E8,
            Square::G8,
            CastlingRights::QueenCastleBlack.index() + CastlingRights::KingCastleBlack.index()
        ),
        CastlingMechanics::new(
            Square::A8,
            Square::D8,
            Square::E8,
            Square::C8,
            CastlingRights::QueenCastleBlack.index() + CastlingRights::KingCastleBlack.index()
        )
    ]
];


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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



#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    const MULTIPLIER: i16;
    const UP: i8;
    const DOWN_RIGHT: i8;
    const DOWN_LEFT: i8;
    const LAST_RANK: Bitboard;
    const DOUBLE_PUSH_RANK: Bitboard;
    type OPPOSITE: Side;

    #[inline(always)]
    fn shift_up(bb: Bitboard) -> Bitboard;

    #[inline(always)]
    fn pawn_attack_pattern_l(bb: Bitboard) -> Bitboard;

    #[inline(always)]
    fn pawn_attack_pattern_r(bb: Bitboard) -> Bitboard;

}

pub struct WhiteSide;
impl Side for WhiteSide {
    const INDEX: usize = 0;
    const OFFSET: usize = 0;
    const MULTIPLIER: i16 = 1;
    const UP: i8 = 8;
    const DOWN_RIGHT: i8 = -7;
    const DOWN_LEFT: i8 = -9;
    const LAST_RANK: Bitboard = RANK_8;
    const DOUBLE_PUSH_RANK: Bitboard = RANK_4;
    
    
    type OPPOSITE = BlackSide;

    #[inline(always)]
    fn shift_up(bb: Bitboard) -> Bitboard {
        bb << 8
    }

    #[inline(always)]
    fn pawn_attack_pattern_l(bb: Bitboard) -> Bitboard {
        (bb << 7) & !FILE_H
    }

    #[inline(always)]
    fn pawn_attack_pattern_r(bb: Bitboard) -> Bitboard {
        (bb << 9) & !FILE_A
    }
    


}

pub struct BlackSide;
impl Side for BlackSide {
    const INDEX: usize = 1;
    const OFFSET: usize = NUM_PIECES as usize;
    const MULTIPLIER: i16 = -1;
    const UP: i8 = -8;
    const DOWN_LEFT: i8 = 7;
    const DOWN_RIGHT: i8 = 9;
    const LAST_RANK: Bitboard = RANK_1;
    const DOUBLE_PUSH_RANK: Bitboard = RANK_5;
    type OPPOSITE = WhiteSide;

    #[inline(always)]
    fn shift_up(bb: Bitboard) -> Bitboard {
        bb >> 8
    }
    
    #[inline(always)]
    fn pawn_attack_pattern_l(bb: Bitboard) -> Bitboard {
        (bb >> 9)  & !FILE_H
    }

    #[inline(always)]
    fn pawn_attack_pattern_r(bb: Bitboard) -> Bitboard {
        
        (bb >> 7) & !FILE_A
    }
}



