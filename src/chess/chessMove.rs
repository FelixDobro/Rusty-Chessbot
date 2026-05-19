use core::fmt;
use std::error::Error;

use crate::chess::{game::Game, square::Square};


pub const MOVE_GEN_SIZE: usize = 256;
pub const GAME_MOVES_SIZE: usize = 1024;




#[derive(Copy, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
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
    pub fn new(from: Square, to: Square, flags: u16) -> Self {
        let from_u = from.u16();
        let to_u = to.u16();
        debug_assert!(from_u < 64 && to_u < 64);
        Self(from_u | (to_u << 6) | (flags << 12))
    }


    #[inline(always)]
    pub fn from_string(m: &str, game: &Game) -> Result<Move, Box<dyn Error>> {
        game.get_board().qualify_move(m)
    }



    #[inline(always)]
    pub fn from(self) -> Square {
        Square::from_u16(self.0 & Move::FROM_MASK)
    }

     #[inline(always)]
    pub fn to_string(self) -> String {
        let uci_move = self.from().to_string() + &self.to().to_string();
        
        let addon = match self.flags() {
                Self::PROMO_CAP_QUEEN => "q",
                Self::PROMO_QUEEN => "q",
                Self::PROMO_CAP_KNIGHT => "n",
                Self::PROMO_KNIGHT => "n",
                Self::PROMO_CAP_BISHOP => "b",
                Self::PROMO_BISHOP => "b",
                Self::PROMO_CAP_ROOK => "r",
                Self::PROMO_ROOK => "r",
                _ => ""
        };
        uci_move + addon
    }

    #[inline(always)]
    pub fn to(self) -> Square {
        Square::from_u16((self.0 & Move::TO_MASK) >> 6)
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
        write!(f, "{}{} {}", self.from(), self.to(), self.flags())
    }
}

#[derive(Debug)]
pub struct MoveList<const N: usize> {
    moves: [Move; N],
    count: usize,
}

impl<const N: usize> MoveList<N> {
    
    pub fn new() -> Self {
        Self {
            moves: [Move(0); N],
            count: 0 
        }
    }

    #[inline(always)]
    pub fn push(&mut self, m: Move) {
        self.moves[self.count] = m;
        self.count += 1;
    } 

    #[inline(always)]
    pub fn pop(&mut self) {
        self.count -= 1;
    }

    pub fn as_slice(&self) -> &[Move] {
        &self.moves[0..self.count]
    }

    pub fn as_mut_slice(&mut self) -> &mut [Move] {
        self.moves[0..self.count].as_mut()
    }

    pub fn print_list(&self) {
        for m in self.as_slice() {
            println!("{}", m)
        }
    }
    
    #[inline(always)]
    pub fn size(&self) -> usize {
        self.count
    }
}



#[cfg(test)]
mod test {



}