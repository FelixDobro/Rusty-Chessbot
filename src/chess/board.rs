#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};
use std::error::Error;
use std::ffi::FromBytesUntilNulError;

use crate::chess::constants::Color::Empty as EmptyC;
use crate::chess::constants::Color::{Black, White};
use crate::chess::constants::Piece::*;
use crate::chess::{chessMove::Move, constants::*};



#[derive(Copy, Clone, Debug)]
#[repr(align(64))]

pub struct Board {
    piece_bb: [u64; 12],
    color_bb: [u64; 2],
    occupied: u64,

    piece: [Piece; 64],
    color: [Color; 64],

    turn: Color,
    en_passant: u64,
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

            color: [
                White, White, White, White, White, White, White, White, White, White, White, White,
                White, White, White, White, EmptyC, EmptyC, EmptyC, EmptyC, EmptyC, EmptyC, EmptyC,
                EmptyC, EmptyC, EmptyC, EmptyC, EmptyC, EmptyC, EmptyC, EmptyC, EmptyC, EmptyC,
                EmptyC, EmptyC, EmptyC, EmptyC, EmptyC, EmptyC, EmptyC, EmptyC, EmptyC, EmptyC,
                EmptyC, EmptyC, EmptyC, EmptyC, EmptyC, Black, Black, Black, Black, Black, Black,
                Black, Black, Black, Black, Black, Black, Black, Black, Black, Black,
            ],

            turn: White,
            en_passant: 0,
        }
    }

    pub fn get_bit_board(&self, i: usize) -> u64 {
        self.piece_bb[i]
    }

    pub fn get_occupied(&self) -> u64 {
        self.occupied
    }

    pub fn knight_pattern(&self, color: Color) -> u64 {
        let board = self.piece_bb[Knight.index() + color.offset()];

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
        let board = self.piece_bb[color.offset() + Piece::King.index()];
        let king_pattern = KING_PATTERNS[board.trailing_zeros() as usize];
        king_pattern
    }

    pub fn king_pseudolegals(&self, color: Color) -> u64 {
        self.king_pattern(color) & !self.color_bb[color.index()]
    }

    pub fn w_single_push(&self) -> u64 {
        let board = self.piece_bb[Color::White.offset() + Piece::Pawn.index()];
        (board << 8) & !self.occupied
    }

    pub fn w_double_push(&self) -> u64 {
        let board = self.w_single_push();
        (board << 8) & !self.occupied & RANK_4
    }

    pub fn w_pawn_attacks(&self) -> u64 {
        let forward = self.piece_bb[White.offset() + Pawn.index()] << 8;
        let mut result = 0u64;
        let black_pieces = self.black_pieces();
        let left_side = (forward << 1) & !FILE_A;
        result |= left_side & black_pieces;

        let right_side = (forward >> 1) & !FILE_H;
        result |= right_side & (black_pieces | self.en_passant);
        result
    }

    pub fn b_single_push(&self) -> u64 {
        let board = self.piece_bb[Color::Black.offset() + Piece::Pawn.index()];
        (board >> 8) & !self.occupied
    }

    pub fn b_double_push(&self) -> u64 {
        let board = self.b_single_push();
        (board >> 8) & !self.occupied & RANK_5
    }

    pub fn b_pawn_attacks(&self) -> u64 {
        let forward = self.piece_bb[Black.offset() + Pawn.index()] >> 8;
        let mut result = 0u64;
        let white_pieces = self.white_pieces();
        let left_side = (forward << 1) & !FILE_A;
        result |= left_side & white_pieces;

        let right_side = (forward >> 1) & !FILE_H;
        result |= right_side & (white_pieces | self.en_passant);
        result
    }

    pub fn rook_pseudolegals(&self, color: Color) -> u64 {
        let mut rooks = self.piece_bb[color.offset() + Rook.index()];
        let mut moves = 0u64;
        while rooks != 0 {
            let sq = rooks.trailing_zeros() as u64;
            let mask = STRAIGHT_LINES[sq as usize];
            let index = unsafe { _pext_u64(self.occupied, mask) };
            moves |= STRAIGHT_LINES_MAGIC[sq as usize][index as usize];
            rooks &= rooks - 1;
        }
        moves & !self.color_bb[color.offset()]
    }

    pub fn bishop_pseudolegals(&self, color: Color) -> u64 {
        let mut bishops = self.piece_bb[color.offset() + Bishop.index()];

        let mut moves = 0u64;
        while bishops != 0 {
            let sq = bishops.trailing_zeros() as u64;
            let mask = DIAGONAL_LINES[sq as usize];
            let index = unsafe { _pext_u64(self.occupied, mask) };
            moves |= DIAG_LINES_MAGIC[sq as usize][index as usize];
            bishops &= bishops - 1;
        }

        moves & !self.color_bb[color.offset()]
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

    pub fn sq_attacked_by(&self, sq: Square, color: Color) -> bool {
        let attacker_pawns = self.piece_bb[color.offset() + Pawn.index()];
        if (PAWN_ATTACKS[color.index() ^ 1][sq.index()] & attacker_pawns) > 0u64 {
            return true;
        }

        let attacker_knights = self.piece_bb[color.offset() + Knight.index()];
        if (KNIGHT_PATTERNS[sq as usize] & attacker_knights) > 0u64 {
            return true;
        }

        let attacking_king = self.piece_bb[color.offset() + King.index()];
        if (KING_PATTERNS[sq as usize] & attacking_king) > 064 {
            return true;
        }

        let attack_bishop_queens = self.piece_bb[color.offset() + Queen.index()]
            | self.piece_bb[color.offset() + Bishop.index()];

        if (self.diag_lines_w_bound(sq) & attack_bishop_queens) > 0u64 {
            return true;
        }

        let attack_rook_queens = self.piece_bb[color.offset() + Queen.index()]
            | self.piece_bb[color.offset() + Rook.index()];

        if (self.straight_lines_w_bound(sq) & attack_rook_queens) > 0u64 {
            return true;
        }

        false
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

    #[inline(always)]
    fn remove_piece(&mut self, sq: Square) {
        let p = self.piece[sq.index()];
        let c = self.color[sq.index()];

        let remove_board = 1u64 << sq.index();

        self.piece_bb[p.index() + c.offset()] ^= remove_board;
        self.occupied ^= remove_board;
        self.color_bb[c.index()] ^= remove_board;

        self.piece[sq.index()] = Empty;
        self.color[sq.index()] = EmptyC;
    }

    #[inline(always)]
    fn remove_piece_boards(&mut self, sq: Square) {
        let p = self.piece[sq.index()];
        let c = self.color[sq.index()];

        let remove_board = 1u64 << sq.index();

        self.piece_bb[p.index() + c.offset()] ^= remove_board;
        self.color_bb[c.index()] ^= remove_board;
    }

    #[inline(always)]
    fn promote(&mut self, from: Square, to: Square, promo: Piece) {
        let from_board = 1u64 << from.index();
        let to_board = 1u64 << to.index();

        let p = self.piece[from.index()];
        let c = self.color[from.index()];

        self.piece_bb[p.index() + c.offset()] ^= to_board;
        self.occupied = self.occupied ^ from_board;

        self.color_bb[c.index()] ^= from_board;

        self.piece[from.index()] = Empty;
        self.piece[to.index()] = promo;

        self.color[from.index()] = EmptyC;
        self.color[to.index()] = c;
    }

    #[inline(always)]
    fn move_piece(&mut self, from: Square, to: Square) {
        let from_board = 1u64 << from.index();
        let to_board = 1u64 << to.index();

        let movement = from_board ^ to_board;
        let p = self.piece[from.index()];
        let c = self.color[from.index()];

        self.piece_bb[p.index() + c.offset()] ^= movement;
        self.occupied = self.occupied ^ from_board;
        self.color_bb[c.index()] ^= from_board;

        self.piece[from.index()] = Empty;
        self.piece[to.index()] = p;

        self.color[from.index()] = EmptyC;
        self.color[to.index()] = c;
    }

    #[inline(always)]
    fn capture(&mut self, from: Square, to: Square) {
        let from_board = 1u64 << from.index();
        let to_board = 1u64 << to.index();

        let movement = from_board ^ to_board;
        let p_capturing = self.piece[from.index()];
        let c_capturing = self.color[from.index()];

        let p_captured = self.piece[to.index()];
        let c_captured = self.color[to.index()];

        // bitboard updates
        self.piece_bb[p_capturing.index() + c_capturing.offset()] ^= movement;
        self.piece_bb[p_captured.index() + c_captured.offset()] ^= to_board;
        

        self.color_bb[c_capturing.index()] ^= from_board;
        self.color_bb[c_capturing.index() ^ 1] ^= to_board;

        self.occupied = self.occupied ^ from_board;

        // 8x8 updates
        self.piece[from.index()] = Empty;
        self.piece[to.index()] = p_capturing;

        self.color[from.index()] = EmptyC;
        self.color[to.index()] = c_capturing;
    }

    #[inline(always)]
    fn capture_promote(&mut self, from: Square, to: Square, promo: Piece) {
        let from_board = 1u64 << from.index();
        let to_board = 1u64 << to.index();

        let p = self.piece[from.index()];
        let c = self.color[from.index()];

        self.piece_bb[p.index() + c.offset()] ^= from_board;
        self.occupied = self.occupied ^ from_board;
        self.color_bb[c.index()] ^= from_board;
        self.color_bb[c.index() ^ 1] ^= to_board;

        self.piece[from.index()] = Empty;
        self.piece[to.index()] = promo;

        self.color[from.index()] = EmptyC;
        self.color[to.index()] = c;
    }

    pub fn make_pseudolegal_move(&mut self, m: Move) {
        match m.flags() {
            Move::QUIET => {
                self.move_piece(Square::from_u8(m.from()), Square::from_u8(m.to()));
                
            }
            Move::KING_CASTLE => match self.turn {
                White => {
                    self.move_piece(Square::E1, Square::G1);
                    self.move_piece(Square::H1, Square::F1);
                }
                Black => {
                    self.move_piece(Square::E8, Square::G8);
                    self.move_piece(Square::H8, Square::F8);
                }
                _ => {}
            },
            Move::QUEEN_CASTLE => match self.turn {
                White => {
                    self.move_piece(Square::E1, Square::C1);
                    self.move_piece(Square::A1, Square::D1);
                }
                Black => {
                    self.move_piece(Square::E8, Square::C8);
                    self.move_piece(Square::A8, Square::D8);
                }
                _ => {}
            },
            Move::EN_PASSANT => {
                self.move_piece(Square::from_u8(m.from()), Square::from_u8(m.to()));
                match self.turn {
                    White => {
                        self.remove_piece(Square::from_u8(m.to() - 8));
                    }
                    Black => {
                        self.remove_piece(Square::from_u8(m.to() + 8));
                    }
                    _ => {}
                }

                self.en_passant = 0;
            }
            Move::DOUBLE_PAWN => {
                let to = m.to();

                match self.turn {
                    White => {
                        self.en_passant = 1u64 << (to - 8);
                    }
                    Black => {
                        self.en_passant = 1u64 << (to + 8);
                    }
                    _ => {}
                }

                self.move_piece(Square::from_u8(m.from()), Square::from_u8(to));
            }

            _ => {
                if m.is_capture() {
                    if m.is_promo() {
                        let promoted_to_piece = match m.flags() {
                            Move::PROMO_CAP_KNIGHT => Knight,
                            Move::PROMO_CAP_QUEEN => Queen,
                            Move::PROMO_CAP_BISHOP => Bishop,
                            Move::PROMO_CAP_ROOK => Rook,
                            _ => Empty,
                        };
                        self.capture_promote(
                            Square::from_u8(m.from()),
                            Square::from_u8(m.to()),
                            promoted_to_piece,
                        );
                    } else {
                        self.capture(Square::from_u8(m.from()), Square::from_u8(m.to()));
                    }
                } else if m.is_promo() {
                    let promoted_to_piece = match m.flags() {
                        Move::PROMO_CAP_KNIGHT => Knight,
                        Move::PROMO_CAP_QUEEN => Queen,
                        Move::PROMO_CAP_BISHOP => Bishop,
                        Move::PROMO_CAP_ROOK => Rook,
                        _ => Empty,
                    };
                    self.promote(
                        Square::from_u8(m.from()),
                        Square::from_u8(m.to()),
                        promoted_to_piece,
                    );
                } else {
                    
                }
            }
        }
        let king_sqaure: u8= self.piece_bb[King.index() + self.turn.offset()].trailing_zeros() as u8;

        if self.sq_attacked_by(Square::from_u8(king_sqaure), self.turn.opposite()) {
            println!("Not legal, unmake would be called");
        }
    }

    pub fn print(&self) -> () {
        println!("  A B C D E F G H");
        for rank in (0..8).rev() {
            print!("{}", rank);
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
}
