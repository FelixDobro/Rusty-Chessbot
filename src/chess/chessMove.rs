use core::fmt;

use crate::chess::constants::Square;

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(transparent)]
pub struct Move(u16);

impl Move {
    const FROM_MASK: u16 = 0x003F;
    const TO_MASK: u16 = 0x0FC0;
    const FLAGMASK: u16 = 0xF000;

    pub const QUIET: u16 = 0;
    pub const DOUBLE_PAWN: u16 = 1;
    pub const KING_CASTLE: u16 = 2;
    pub const QUEEN_CASTLE: u16 = 3;
    pub const CAPTURE: u16 = 4;
    pub const EN_PASSANT: u16 = 5;
    pub const PROMO_KNIGHT: u16 = 8;
    pub const PROMO_BISHOP: u16 = 9;
    pub const PROMO_ROOK: u16 = 10;
    pub const PROMO_QUEEN: u16 = 11;
    pub const PROMO_CAP_KNIGHT: u16 = 12;
    pub const PROMO_CAP_BISHOP: u16 = 13;
    pub const PROMO_CAP_ROOK: u16 = 14;
    pub const PROMO_CAP_QUEEN: u16 = 15;

    #[inline(always)]
    pub fn new(from: u16, to: u16, flags: u16) -> Self {
        debug_assert!(from < 64 && to < 64);
        Self(from | (to << 6) | (flags << 12))
    }


    #[inline(always)]
    pub fn from(self) -> u8 {
        (self.0 & Move::FROM_MASK) as u8
    }

    #[inline(always)]
    pub fn to(self) -> u8 {
        ((self.0 & Move::TO_MASK) >> 6) as u8
    }

    #[inline(always)]
    pub fn flags(self) -> u16 {
        (self.0 & Move::FLAGMASK) >> 12
    }


    #[inline(always)]
    pub fn is_quiet(self) -> bool {
        self.flags() == 0 
    }

    #[inline(always)]
    pub fn is_capture(self) -> bool {
        (self.flags() & 4) != 0 
    }

    #[inline(always)]
    pub fn is_simple_promo(self) -> bool {
        self.flags() < 12
    }

    #[inline(always)]
    pub fn is_promo(self) -> bool {
        (self.flags() & 8) != 0
    }

    #[inline(always)]
    pub fn is_castle(&self) -> bool {
        self.flags() == 3 || self.flags() == 2
    }

    #[inline(always)]
    pub fn get_castle_idx(&self) -> usize {
        self.flags().trailing_zeros() as usize
    }

}


impl fmt::Display for Move {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?} {}", Square::from_u8(self.from()), Square::from_u8(self.to()), self.flags())
    }
}

const MAX_MOVES: usize = 256;

pub struct MoveList {

    moves: [Move; MAX_MOVES],
    count: usize,
}

impl MoveList {
    
    pub fn new() -> Self {
        Self { moves: [Move(0); MAX_MOVES], count: 0 }
    }

    #[inline(always)]
    pub fn push(&mut self, m: Move) {
        self.moves[self.count] = m;
        self.count += 1;
    } 

    pub fn as_sclice(&self) -> &[Move] {
        &self.moves[0..self.count]
    }

    pub fn print_list(&self) {
        for m in self.as_sclice() {
            println!("{}", m)
        }
    }
    
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.count
    }
}