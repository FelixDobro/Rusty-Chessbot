pub mod move_gen;
pub mod hash;
pub mod square;
pub mod chessMove;
pub mod bitboard;
pub mod constants;


#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};
use crate::chess::bitboard::{*};
use crate::chess::bitboard::EMPTY as EMPTY_BB;

use crate::chess::square::Square;

use crate::chess::constants::{*};
use crate::chess::constants::Color::{Black, White};
use crate::chess::constants::{Side, BlackSide, WhiteSide, Piece, CastlingRights, Color};
use crate::chess::constants::Piece::{*};


#[derive(Copy, Clone, Debug)]
#[repr(align(64))]

pub struct Board {
    piece_bb: [Bitboard; 12],
    color_bb: [Bitboard; 2],
    occupied: Bitboard,

    piece: [Piece; 64],

    pub turn: Color,
    pub en_passant: Bitboard,
    pub castling_rights: u8,
    halfmoves: u8,
    hash: u64
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
        let mut board = Board {
            piece_bb: [
                PAWN_W_DEFAULT,
                BISHOP_W_DEFAULT,
                KNIGHT_W_DEFAULT,
                ROOK_W_DEFAULT,
                QUEEN_W_DEFAULT,
                KING_W_DEFAULT,
                PAWN_B_DEFAULT,
                BISHOP_B_DEFAULT,
                KNIGHT_B_DEFAULT,
                ROOK_B_DEFAULT,
                QUEEN_B_DEFAULT,
                KING_B_DEFAULT,
            ],

            color_bb: [DEFAULT_COLOR_W, DEFAULT_COLOR_B],

            occupied: DEFAULT_OCCUPIED,

            piece: [
                Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook, Pawn, Pawn, Pawn, Pawn,
                Pawn, Pawn, Pawn, Pawn, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                Pawn, Pawn, Pawn, Pawn, Pawn, Pawn, Pawn, Pawn, Rook, Knight, Bishop, Queen, King,
                Bishop, Knight, Rook,
            ],

            turn: White,
            en_passant: EMPTY_BB,
            castling_rights: 0xF,
            halfmoves: 0,
            hash: 0
        };
        board.hash = board.calculate_hash();
        board
    }


    pub fn from_fen(fen_string: &str) -> Result<Board, FenError> {
        let mut piece_bb = [EMPTY_BB; 12];
        let mut piece_8_board = [Empty; 64];
        let mut color_bb = [EMPTY_BB; 2];
        let mut occupied = EMPTY_BB;
        let mut turn: Color;
        let mut en_passant_right = EMPTY_BB;
        let mut castling_rights = 0;
        let mut halfmoves_b: u8 = 0;
        let mut fullmoves_b = 0; 


        let mut splitted= fen_string.split(" ");

        if let Some(ranks) = splitted.next() {
            let mut num_ranks = 0;

            if num_ranks > 7 {return Err(FenError::InvalidNumRanks)}
        
            for rank in ranks.split("/") {
                let mut square_offset: u8 = (7 - num_ranks) * 8;
                for c in rank.chars(){ 
                    if let Some(number) = c.to_digit(10) {
                        square_offset += number as u8; 
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

                        let appears_board = Square::from_u8(square_offset).to_bitboard();
    
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
                    'K' => CastlingRights::KingCastleWhite as u8,
                    'Q' => CastlingRights::QueenCastleWhite as u8,
                    'k' => CastlingRights::KingCastleBlack as u8,
                    'q' => CastlingRights::QueenCastleBlack as u8,
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
                "-" => EMPTY_BB,
                _ => {
                    if let Ok(square) = Square::from_string(en_passant) {
                        square.to_bitboard()
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

        let mut board = Board { piece_bb, color_bb, occupied, piece: piece_8_board, turn, en_passant: en_passant_right, castling_rights, halfmoves: halfmoves_b, hash: 0};
        board.hash = board.calculate_hash();  
        Ok(board)
    }

    #[inline(always)]
    pub fn get_hash(&self) -> u64 {
        self.hash
    }

    #[inline(always)]
    pub fn get_enpassant(&self) -> Bitboard {
        self.en_passant
    }


    pub fn get_bit_board(&self, i: usize) -> Bitboard {
        self.piece_bb[i]
    }

    pub fn get_occupied(&self) -> Bitboard {
        self.occupied
    }

    #[inline(always)]
    pub fn knight_pattern<S: Side>(&self) -> Bitboard {
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

    pub fn black_pieces(&self) -> Bitboard {
        self.color_bb[BlackSide::INDEX]
    }

    pub fn white_pieces(&self) -> Bitboard {
        self.color_bb[WhiteSide::INDEX]
    }

    pub fn knight_pseudolegals<S: Side>(&self) -> Bitboard {
        self.knight_pattern::<S>() & !self.color_bb[S::INDEX]
    }

    pub fn king_pattern<S: Side>(&self) -> Bitboard {
        let board = self.piece_bb[S::OFFSET + King.index()];
        let king_pattern = KING_PATTERNS[board.lsb().usize()];
        king_pattern
    }

    pub fn king_pseudolegals<S: Side>(&self) -> Bitboard {
        self.king_pattern::<S>() & !self.color_bb[S::INDEX]
    }

    pub fn pawn_single_push<S: Side>(&self) -> Bitboard {
        let board = self.piece_bb[S::OFFSET + Pawn.index()];
        S::shift_up(board) & !self.occupied
    }

    pub fn pawn_double_push<S: Side>(&self) -> Bitboard {
        let board = self.pawn_single_push::<S>();
        S::shift_up(board) & !self.occupied
    }

    pub fn w_pawn_attacks(&self) -> Bitboard {
        let forward = self.piece_bb[White.offset() + Pawn.index()] << 8;
        let mut result = EMPTY_BB;
        let black_pieces = self.black_pieces();
        let left_side = (forward << 1) & !FILE_A;
        result |= left_side & (black_pieces | self.en_passant);

        let right_side = (forward >> 1) & !FILE_H;
        result |= right_side & (black_pieces | self.en_passant);
        result
    }

    pub fn b_pawn_attacks(&self) -> Bitboard {
        let forward = self.piece_bb[Black.offset() + Pawn.index()] >> 8;
        let mut result = EMPTY_BB;
        let white_pieces = self.white_pieces();
        let left_side = (forward << 1) & !FILE_A;
        result |= left_side & (white_pieces | self.en_passant);

        let right_side = (forward >> 1) & !FILE_H;
        result |= right_side & (white_pieces | self.en_passant);
        result
    }

    pub fn rook_pseudolegals<S: Side>(&self) -> Bitboard {
        let mut rooks = self.piece_bb[S::OFFSET + Rook.index()];
        let mut moves = EMPTY_BB;
        while rooks != EMPTY_BB {
            let sq = rooks.lsb().u8();
            let mask = STRAIGHT_LINES[sq as usize];
            let index = unsafe { _pext_u64(self.occupied.u64(), mask.u64()) };
            moves |= STRAIGHT_LINES_MAGIC[sq as usize][index as usize];
            rooks.pop_lsb();
        }
        moves & !self.color_bb[S::INDEX]
    }

    pub fn bishop_pseudolegals<S: Side>(&self) -> Bitboard {
        let mut bishops = self.piece_bb[S::OFFSET + Bishop.index()];

        let mut moves = EMPTY_BB;
        while bishops != EMPTY_BB {
            let sq = bishops.lsb();
            let mask = DIAGONAL_LINES[sq.usize()];
            let index = unsafe { _pext_u64(self.occupied.u64(), mask.u64()) };
            moves |= DIAG_LINES_MAGIC[sq.usize()][index as usize];
            bishops.pop_lsb();
        }

        moves & !self.color_bb[S::INDEX]
    }

    pub fn diag_lines_w_bound(&self, sq: Square) -> Bitboard {
        let mask = DIAGONAL_LINES[sq.usize()];
        let index = unsafe { _pext_u64(self.occupied.u64(), mask.u64()) };
        DIAG_LINES_MAGIC[sq.usize()][index as usize]
    }

    pub fn straight_lines_w_bound(&self, sq: Square) -> Bitboard {
        let mask = STRAIGHT_LINES[sq.usize()];
        let index = unsafe { _pext_u64(self.occupied.u64(), mask.u64()) };
        STRAIGHT_LINES_MAGIC[sq.usize()][index as usize]
    }

    pub fn sq_attacked_by<S: Side>(&self, sq: Square) -> bool {
        let attacker_pawns = self.piece_bb[S::OFFSET + Pawn.index()];
        if (PAWN_ATTACKS[S::OPPOSITE::INDEX][sq.index()] & attacker_pawns) > EMPTY_BB {

            return true;
        }

        let attacker_knights = self.piece_bb[S::OFFSET + Knight.index()];
        if (KNIGHT_PATTERNS[sq.usize()] & attacker_knights) > EMPTY_BB {

            return true;
        }

        let attacking_king = self.piece_bb[S::OFFSET + King.index()]; 
        
        if (KING_PATTERNS[sq.usize()] & attacking_king) > EMPTY_BB {
            
            return true;
        }

        let attack_bishop_queens =
            self.piece_bb[S::OFFSET + Queen.index()] | self.piece_bb[S::OFFSET + Bishop.index()];

        if (self.diag_lines_w_bound(sq) & attack_bishop_queens) > EMPTY_BB {

            return true;
        }

        let attack_rook_queens =
            self.piece_bb[S::OFFSET + Queen.index()] | self.piece_bb[S::OFFSET + Rook.index()];

        if (self.straight_lines_w_bound(sq) & attack_rook_queens) > EMPTY_BB {


            return true;
        }

        false
    }

    #[inline(always)]
    pub fn get_king_square<S: Side>(&self) -> Square {

        Square::from_u8(self.piece_bb[S::OFFSET + King.index()].lsb().u8())
    }

    pub fn get_piece(&self, square: Square) -> Piece {
        self.piece[square.usize()]
    }

    pub fn get_piece_usize(&self, square: Square) -> usize {
        return self.get_piece(square) as usize;
    }

    pub fn get_color(&self, square: Square) -> Color {
        if self.piece_bb[0] & (square.to_bitboard()) != EMPTY_BB {
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
                        let color = self.get_color(Square::from_u8(rank * 8 + file));

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
            self.piece_bb[i + color.offset()].print();
        }
    }

}