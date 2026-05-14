mod move_gen;
mod hash;

use core::num;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};
use std::fmt::Error;
use crate::chess::constants::{*};

use crate::chess::constants::Color::{Black, White};
use crate::chess::constants::Piece::*;


#[derive(Copy, Clone, Debug)]
#[repr(align(64))]

pub struct Board {
    piece_bb: [u64; 12],
    color_bb: [u64; 2],
    occupied: u64,

    piece: [Piece; 64],

    pub turn: Color,
    pub en_passant: u64,
    pub castling_rights: u8,
    halfmoves: u8,
    fullmoves: u16,
}


#[derive(Debug)]
pub enum FenError {
    InvalidNumSections,
    InvalidTurn,
    InvalidNumRanks,
    InvalidPiece,
    Castling,
    EnPassant,
    HalfMove,
    FullMove,
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

            piece: [
                Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook, Pawn, Pawn, Pawn, Pawn,
                Pawn, Pawn, Pawn, Pawn, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Pawn, Pawn, Pawn, Pawn, Pawn, Pawn, Pawn, Pawn, Rook, Knight, Bishop, Queen, King,
                Bishop, Knight, Rook,
            ],

            turn: White,
            en_passant: 0,
            castling_rights: 0xF,
            fullmoves: 1,
            halfmoves: 0
        }
    }


    pub fn from_fen(fen_string: &str) -> Result<Board, FenError> {
        let mut piece_bb = [0u64; 12];
        let mut piece_8_board = [Empty; 64];
        let mut color_bb = [0u64; 2];
        let mut occupied = 0u64;
        let mut turn: Color;
        let mut en_passant_right = 0u64;
        let mut castling_rights = 0;
        let mut halfmoves_b: u8 = 0;
        let mut fullmoves_b = 0; 


        let mut splitted= fen_string.split(" ");

        if let Some(ranks) = splitted.next() {
            let mut num_ranks = 0;

            if num_ranks > 7 {return Err(FenError::InvalidNumRanks)}
        
            for rank in ranks.split("/") {
                let mut square_offset = (7 - num_ranks) * 8;
                for c in rank.chars(){ 
                    if let Some(number) = c.to_digit(10) {
                        square_offset += number; 
                    }
                    else {
                        let color = if c.is_uppercase() {White} else {Black};

                        let piece = match c.to_ascii_lowercase() {
                            'p' => Pawn,
                            'n' => Knight,
                            'b' => Bishop,
                            'r' => Rook,
                            'q' => Queen,
                            'k' => King,
                            _ => return Err(FenError::InvalidPiece)
                        };

                        let appears_board = 1u64 << square_offset;
    
                        piece_8_board[square_offset as usize] = piece;
                        piece_bb[piece.index() + color.offset()] ^= appears_board;
                        color_bb[color.index()] ^= appears_board;  
                        occupied ^= appears_board;
                        square_offset += 1;

                    }
                }
                num_ranks += 1;
            } 
            if  num_ranks != 8 {
                return Err(FenError::InvalidNumRanks);
            }
        } 
        else {
            return Err(FenError::InvalidNumSections)
        }

        if let Some(side_to_move) = splitted.next() {
            turn = match side_to_move {
                "w" => White,
                "b" => Black,
                _ => return Err(FenError::InvalidTurn)
            };
        }
        else {
            return Err(FenError::InvalidNumSections)
        }

        if let Some(rights) = splitted.next() {
            
            for right in rights.chars() {
                castling_rights += match right {
                    'K' => CASTLING_RIGHTS::KingCastleWhite as u8,
                    'Q' => CASTLING_RIGHTS::QueenCastleWhite as u8,
                    'k' => CASTLING_RIGHTS::KingCastleBlack as u8,
                    'q' => CASTLING_RIGHTS::QueenCastleBlack as u8,
                    '-' => 0,
                    _ => return Err(FenError::Castling)
                }
            }
        } 
        else {
            return Err(FenError::InvalidNumSections);
        }

        if let Some(en_passant) = splitted.next() {
            en_passant_right = match en_passant {
                "-" => 0,
                _ => {
                    if let Ok(square) = Square::from_string(en_passant) {
                        1u64 << (square as u8)
                    }
                    else {
                        return Err(FenError::EnPassant)
                    }
                }
            };
        }

        if let Some(halfmoves) = splitted.next() {
            if let Ok(num) = halfmoves.parse::<u8>() {
                halfmoves_b = num
            }
            else {
                return Err(FenError::HalfMove);
            }
        }
        else {
            return Err(FenError::InvalidNumSections)
        }
    
        if let Some(fullmove) = splitted.next() {
            if let Ok(num) = fullmove.parse::<u16>() {
                fullmoves_b = num;
            }
            else {
                return Err(FenError::FullMove);
            }
        }
        else {
            return Err(FenError::InvalidNumSections)
        }

        Ok(Board { piece_bb, color_bb, occupied, piece: piece_8_board, turn, en_passant: en_passant_right, castling_rights, halfmoves: halfmoves_b, fullmoves: fullmoves_b})

    }


    pub fn get_bit_board(&self, i: usize) -> u64 {
        self.piece_bb[i]
    }

    pub fn get_occupied(&self) -> u64 {
        self.occupied
    }

    #[inline(always)]
    pub fn knight_pattern<S: Side>(&self) -> u64 {
        let board = self.piece_bb[Knight.index() + S::OFFSET];

        // 2 horizontal 1 vertical
        let mut prefix = (board << 16) | (board >> 16);
        let first_half = ((prefix << 1) & !FILE_A) | ((prefix >> 1) & !FILE_H);

        // 1 horizontal 2 vertical
        prefix = (board << 8) | (board >> 8);

        let final_pattern = ((prefix >> 2) & !(FILE_G | FILE_H))
            | ((prefix << 2) & !(FILE_A | FILE_B))
            | (first_half);
        final_pattern & !self.color_bb[S::INDEX]
    }

    pub fn black_pieces(&self) -> u64 {
        self.color_bb[BlackSide::INDEX]
    }

    pub fn white_pieces(&self) -> u64 {
        self.color_bb[WhiteSide::INDEX]
    }

    pub fn knight_pseudolegals<S: Side>(&self) -> u64 {
        self.knight_pattern::<S>() & !self.color_bb[S::INDEX]
    }

    pub fn king_pattern<S: Side>(&self) -> u64 {
        let board = self.piece_bb[S::OFFSET + King.index()];
        let king_pattern = KING_PATTERNS[board.trailing_zeros() as usize];
        king_pattern
    }

    pub fn king_pseudolegals<S: Side>(&self) -> u64 {
        self.king_pattern::<S>() & !self.color_bb[S::INDEX]
    }

    pub fn pawn_single_push<S: Side>(&self) -> u64 {
        let board = self.piece_bb[S::OFFSET + Pawn.index()];
        S::shift_up(board) & !self.occupied
    }

    pub fn pawn_double_push<S: Side>(&self) -> u64 {
        let board = self.pawn_single_push::<S>();
        S::shift_up(board) & !self.occupied
    }

    pub fn w_pawn_attacks(&self) -> u64 {
        let forward = self.piece_bb[White.offset() + Pawn.index()] << 8;
        let mut result = 0u64;
        let black_pieces = self.black_pieces();
        let left_side = (forward << 1) & !FILE_A;
        result |= left_side & (black_pieces | self.en_passant);

        let right_side = (forward >> 1) & !FILE_H;
        result |= right_side & (black_pieces | self.en_passant);
        result
    }

    pub fn b_pawn_attacks(&self) -> u64 {
        let forward = self.piece_bb[Black.offset() + Pawn.index()] >> 8;
        let mut result = 0u64;
        let white_pieces = self.white_pieces();
        let left_side = (forward << 1) & !FILE_A;
        result |= left_side & (white_pieces | self.en_passant);

        let right_side = (forward >> 1) & !FILE_H;
        result |= right_side & (white_pieces | self.en_passant);
        result
    }

    pub fn rook_pseudolegals<S: Side>(&self) -> u64 {
        let mut rooks = self.piece_bb[S::OFFSET + Rook.index()];
        let mut moves = 0u64;
        while rooks != 0 {
            let sq = rooks.trailing_zeros() as u64;
            let mask = STRAIGHT_LINES[sq as usize];
            let index = unsafe { _pext_u64(self.occupied, mask) };
            moves |= STRAIGHT_LINES_MAGIC[sq as usize][index as usize];
            rooks &= rooks - 1;
        }
        moves & !self.color_bb[S::INDEX]
    }

    pub fn bishop_pseudolegals<S: Side>(&self) -> u64 {
        let mut bishops = self.piece_bb[S::OFFSET + Bishop.index()];

        let mut moves = 0u64;
        while bishops != 0 {
            let sq = bishops.trailing_zeros() as u64;
            let mask = DIAGONAL_LINES[sq as usize];
            let index = unsafe { _pext_u64(self.occupied, mask) };
            moves |= DIAG_LINES_MAGIC[sq as usize][index as usize];
            bishops &= bishops - 1;
        }

        moves & !self.color_bb[S::INDEX]
    }

    pub fn diag_lines_w_bound(&self, sq: Square) -> u64 {
        let mask = DIAGONAL_LINES[sq as usize];
        let index = unsafe { _pext_u64(self.occupied, mask) };
        DIAG_LINES_MAGIC[sq as usize][index as usize]
    }

    pub fn straight_lines_w_bound(&self, sq: Square) -> u64 {
        let mask = STRAIGHT_LINES[sq as usize];
        let index = unsafe { _pext_u64(self.occupied, mask) };
        STRAIGHT_LINES_MAGIC[sq as usize][index as usize]
    }

    pub fn sq_attacked_by<S: Side>(&self, sq: Square) -> bool {
        let attacker_pawns = self.piece_bb[S::OFFSET + Pawn.index()];
        if (PAWN_ATTACKS[S::OPPOSITE::INDEX][sq.index()] & attacker_pawns) > 0u64 {

            return true;
        }

        let attacker_knights = self.piece_bb[S::OFFSET + Knight.index()];
        if (KNIGHT_PATTERNS[sq as usize] & attacker_knights) > 0u64 {

            return true;
        }

        let attacking_king = self.piece_bb[S::OFFSET + King.index()]; 
        
        if (KING_PATTERNS[sq as usize] & attacking_king) > 0u64 {
            
            return true;
        }

        let attack_bishop_queens =
            self.piece_bb[S::OFFSET + Queen.index()] | self.piece_bb[S::OFFSET + Bishop.index()];

        if (self.diag_lines_w_bound(sq) & attack_bishop_queens) > 0u64 {

            return true;
        }

        let attack_rook_queens =
            self.piece_bb[S::OFFSET + Queen.index()] | self.piece_bb[S::OFFSET + Rook.index()];

        if (self.straight_lines_w_bound(sq) & attack_rook_queens) > 0u64 {


            return true;
        }

        false
    }

    #[inline(always)]
    pub fn get_king_square<S: Side>(&self) -> Square {

        Square::from_u8(self.piece_bb[S::OFFSET + King.index()].trailing_zeros() as u8)
    }

    pub fn get_piece(&self, square: Square) -> Piece {
        self.piece[square as usize]
    }

    pub fn get_piece_usize(&self, square: Square) -> usize {
        return self.get_piece(square) as usize;
    }

    pub fn get_color(&self, square: u8) -> Color {
        if self.piece_bb[0] & (1u64 << square) != 0 {
            return White;
        } else {
            return Black;
        }
    }


    pub fn print(&self) -> () {
        println!("  A B C D E F G H");
        for rank in (0..8).rev() {
            print!("{}", rank + 1);
            for file in 0..8 {
                let piece = self.get_piece(Square::from_u8(rank * 8 + file));

                match piece {
                    Empty => print!(" ."),
                    _ => {
                        let color = self.get_color(rank * 8 + file);

                        let symbol = SYMBOLS[piece.index() + color.offset()];
                        print!(" {}", symbol.to_string());
                    }
                }
            }
            println!();
        }

        println!("  A B C D E F G H");
    }


    pub fn print_bitboards(&self, color: Color) {
        for i in 0..6 {
            print_bitboard(self.piece_bb[i + color.offset()]);
        }
    }

}