use std::error::Error;

use crate::chess::constants::Color::Empty as EmtypyC;
use crate::chess::constants::Color::{Black, White};
use crate::chess::constants::Piece::*;
use crate::chess::constants::*;

#[derive(Copy, Clone, Debug)]
#[repr(align(64))]

pub struct Board {
    piece_bb: [u64; 12],
    color_bb: [u64; 2],
    occupied: u64,
    empty: u64,

    piece: [Piece; 64],
    color: [Color; 64],
}

impl Board {
    pub fn default() -> Self {
        Board {
            piece_bb: [
                0x000000000000FF00,
                0x0000000000000024,
                0x0000000000000042,
                0x0000000000000081,
                0x0000000000000008,
                0x0000000000000010,
                0x00FF000000000000,
                0x2400000000000000,
                0x4200000000000000,
                0x8100000000000000,
                0x0800000000000000,
                0x1000000000000000,
            ],

            color_bb: [0x000000000000FFFF, 0xFFFF000000000000],
            
            occupied: 0x000000000000FFFF | 0xFFFF000000000000,
            empty: !(0x000000000000FFFF | 0xFFFF000000000000),

            piece: [
                Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook, Pawn, Pawn, Pawn, Pawn,
                Pawn, Pawn, Pawn, Pawn, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Pawn, Pawn, Pawn, Pawn, Pawn, Pawn, Pawn, Pawn, Rook, Knight, Bishop, Queen, King,
                Bishop, Knight, Rook,
            ],

            color: [
                White, White, White, White, White, White, White, White, White, White, White, White,
                White, White, White, White, EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC,
                EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC,
                EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC,
                EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC, EmtypyC, Black,
                Black, Black, Black, Black, Black, Black, Black, Black, Black, Black, Black, Black,
                Black, Black, Black,
            ],
        }
    }

    pub fn get_bit_board(&self, i: usize) -> u64 {
        self.piece_bb[i]
    }

    pub fn get_occupied(&self) -> u64 {
        self.occupied
    }

    pub fn knight_pattern(&self, color: Color) -> u64 {
        let board = self.piece_bb[Knight.index() + NUM_PIECES * color.index()];

        // 2 horizontal 1 vertical
        let mut prefix = (board << 16) | (board >> 16);
        let first_half = ((prefix << 1) & !FILE_A) | ((prefix >> 1) & !FILE_H);

        // 1 horizontal 2 vertical
        prefix = (board << 8) | (board >> 8);

        let final_pattern = ((prefix >> 2) & !(FILE_G | FILE_H))
            | ((prefix << 2) & !(FILE_A | FILE_B))
            | (first_half);
        final_pattern & !self.color_bb[color.index()]
    }

    pub fn black_pieces(&self) -> u64 {
        self.color_bb[Color::Black.index()]
    }

    pub fn white_pieces(&self) -> u64 {
        self.color_bb[Color::White.index()]
    }

    pub fn knight_pseudolegals(&self, color: Color) -> u64 {
        self.knight_pattern(color) & !self.color_bb[color.index()]
    }

    pub fn king_pattern(&self, color: Color) -> u64 {
        let board = self.piece_bb[color.index() * NUM_PIECES + Piece::King.index()];
        let king_pattern = KING_PATTERNS[board.trailing_zeros() as usize];
        king_pattern
    }

    pub fn king_pseudolegals(&self, color: Color) -> u64 {
        self.king_pattern(color) & !self.color_bb[color.index()]
    }

    pub fn w_single_push(&self) -> u64 {
        let board = self.piece_bb[Color::White.index() * NUM_PIECES + Piece::Pawn.index()];
        (board << 8) & self.empty
    }

    pub fn w_double_push(&self) -> u64 {
        let board = self.w_single_push();
        (board << 8) & self.empty & RANK_4
    }

    pub fn b_single_push(&self) -> u64 {
        let board = self.piece_bb[Color::Black.index() * NUM_PIECES + Piece::Pawn.index()];
        (board >> 8) & self.empty
    }

    pub fn b_double_push(&self) -> u64 {
        let board = self.b_single_push();
        (board >> 8) & self.empty & RANK_5
    }


    


    pub fn get_piece(&self, square: Square) -> Piece {
        self.piece[square as usize]
    }

    pub fn get_piece_usize(&self, square: Square) -> usize {
        return self.get_piece(square) as usize;
    }

    pub fn get_color(&self, square: Square) -> Color {
        self.color[square as usize]
    }

    pub fn get_color_usize(&self, square: Square) -> usize {
        self.color[square as usize] as usize
    }

    pub fn print(&self) -> Result<(), Box<dyn Error>> {
        let mut s = "".to_string();
        s = s + " A B C D E F G H";
        s += "\n";

        for i in (0..64).rev() {
            let piece = self.get_piece_usize(Square::try_from(i)?);

            if piece == Empty as usize {
                s += " ."
            } else {
                let color = self.get_color_usize(Square::try_from(i)?);

                let symbol = SYMBOLS[piece + color * NUM_PIECES];
                s += " ";
                s += &symbol.to_string();
            }
            if i % 8 == 0 {
                s += "\n";
            }
        }

        s += " A B C D E F G H";
        println!("{}", s);
        Ok(())
    }
}
