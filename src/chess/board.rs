#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};
use std::backtrace::Backtrace;
use std::error::Error;
use std::ffi::FromBytesUntilNulError;
use std::io::StdoutLock;

use crate::chess::chessMove::MoveList;
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


    turn: Color,
    pub en_passant: u64,
    castling_rights: u8,
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
            castling_rights: 0xF
        }
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
        if (KING_PATTERNS[sq as usize] & attacking_king) > 064 {
            return true;
        }

        let attack_bishop_queens = self.piece_bb[S::OFFSET + Queen.index()]
            | self.piece_bb[S::OFFSET + Bishop.index()];

        if (self.diag_lines_w_bound(sq) & attack_bishop_queens) > 0u64 {
            return true;
        }

        let attack_rook_queens = self.piece_bb[S::OFFSET + Queen.index()]
            | self.piece_bb[S::OFFSET + Rook.index()];

        if (self.straight_lines_w_bound(sq) & attack_rook_queens) > 0u64 {
            return true;
        }

        false
    }

    #[inline(always)]
    fn get_king_square<S: Side>(&self) -> Square {
        Square::from_u8(self.piece_bb[S::OFFSET + King.index()].trailing_zeros() as u8)
    }

    pub fn get_piece(&self, square: Square) -> Piece {
        self.piece[square as usize]
    }

    pub fn get_piece_usize(&self, square: Square) -> usize {
        return self.get_piece(square) as usize;
    }

    pub fn generate_pseudolegals(&self) -> MoveList {
        match self.turn {
            White => self.pseudolegal_moves::<WhiteSide>(),
            Black => self.pseudolegal_moves::<BlackSide>(),
            _ => panic!()
        }
    }
  
    fn pseudolegal_moves<S: Side>(&self) -> MoveList {
        let mut move_list = MoveList::new();
        self.pawn_moves::<S>(&mut move_list);
        self.knigh_moves::<S>(&mut move_list);
        move_list
    }

    pub fn knigh_moves<S:Side>(&self, move_list: &mut MoveList) {
        let mut knight_board = self.piece_bb[S::OFFSET + Knight.index()];
        
        while knight_board != 0 {
            let from_sqaure = knight_board.trailing_zeros() as u16;
            
            let pattern_board = KNIGHT_PATTERNS[from_sqaure as usize];
            let mut captures = pattern_board & self.color_bb[S::OPPOSITE::INDEX];
            let mut normal_moves = pattern_board & !self.color_bb[S::OPPOSITE::INDEX] & !self.color_bb[S::INDEX];
            
            while captures != 0 {
                let to_square = captures.trailing_zeros() as u16;
                move_list.push(Move::new(from_sqaure, to_square, Move::CAPTURE));
                captures &= captures -1;
            }

            while normal_moves != 0 {
                let to_square = normal_moves.trailing_zeros() as u16;
                move_list.push(Move::new(from_sqaure, to_square, Move::QUIET));
                normal_moves &= normal_moves -1;
            }
            knight_board &= knight_board -1;
        }
    }

    #[inline(always)]
    pub fn pawn_moves<S:Side>(&self, move_list: &mut MoveList) {
        let pawn_board = self.piece_bb[Pawn.index() + S::OFFSET];

        let mut pushes = S::shift_up(pawn_board) & !self.occupied;
        let mut single_pushes = pushes &! S::LAST_RANK;
        let mut double_pushes = S::shift_up(single_pushes) & S::DOUBLE_PUSH_RANK & !self.occupied;
        let mut promotions = pushes & S::LAST_RANK;

        let attack_pattern_left = S::pawn_attack_pattern_l(pawn_board);
        let attack_pattern_right =  S::pawn_attack_pattern_r(pawn_board);

        let all_left_attacks = attack_pattern_left & self.color_bb[S::OPPOSITE::INDEX];
        let mut left_attacks= all_left_attacks & !S::LAST_RANK;
        let mut left_promotions_cap = all_left_attacks & S::LAST_RANK;

        let all_right_attacks = attack_pattern_right & self.color_bb[S::OPPOSITE::INDEX];
        let mut right_attacks = all_right_attacks & !S::LAST_RANK;
        let mut right_promotions_cap = S::LAST_RANK & right_attacks;

        let mut left_en_passants = attack_pattern_left & self.en_passant;
        let mut right_en_passants = attack_pattern_right & self.en_passant;
        

        while single_pushes != 0 {
            let to_square = single_pushes.trailing_zeros() as i8; 
            move_list.push(Move::new((to_square - S::UP) as u16, to_square as u16, 0));
            single_pushes &= single_pushes-1;
        }

        while double_pushes != 0 {
            let to_square = double_pushes.trailing_zeros() as i8;
            move_list.push(Move::new((to_square - S::UP - S::UP) as u16, to_square as u16, 1));
            double_pushes &= double_pushes -1;
        }

        while promotions != 0 {
            let to_square = promotions.trailing_zeros() as i8;
            let from_square = (to_square - S::UP) as u16;
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_KNIGHT));
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_QUEEN));
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_ROOK));
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_BISHOP));
            promotions &= promotions -1;
        }

        while left_attacks != 0 {
            let to_square = left_attacks.trailing_zeros() as i8;
            move_list.push(Move::new((to_square - S::UP - 1) as u16, to_square as u16, Move::CAPTURE));
            left_attacks &= left_attacks -1;
        }

        while right_attacks != 0 {
            let to_square = right_attacks.trailing_zeros() as i8;
            move_list.push(Move::new((to_square - S::UP + 1) as u16, to_square as u16, Move::CAPTURE));
            right_attacks &= right_attacks -1;
        }

        while left_en_passants != 0 {
            let to_square = left_en_passants.trailing_zeros() as i8;
            move_list.push(Move::new((to_square - S::UP - 1) as u16, to_square as u16, Move::EN_PASSANT));
            left_en_passants &= left_en_passants -1;
        }

        while right_en_passants != 0 {
            let to_square = right_en_passants.trailing_zeros() as i8;
            move_list.push(Move::new((to_square - S::UP + 1) as u16, to_square as u16, Move::EN_PASSANT));
            right_en_passants &= right_en_passants -1;   
        }

        while left_promotions_cap != 0 {
            let to_square = left_promotions_cap.trailing_zeros() as i8;
            let from_square =(to_square - S::UP - 1) as u16;
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_CAP_KNIGHT));
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_CAP_BISHOP));
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_CAP_QUEEN));
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_CAP_ROOK));
            left_promotions_cap &= left_promotions_cap -1;
        }

        while right_promotions_cap != 0 {
            let to_square = right_promotions_cap.trailing_zeros() as i8;
            let from_square =(to_square - S::UP + 1) as u16;
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_CAP_KNIGHT));
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_CAP_BISHOP));
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_CAP_QUEEN));
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_CAP_ROOK));
            right_promotions_cap &= right_promotions_cap -1;
        }

    }

    #[inline(always)]
    fn promote<S: Side>(&mut self, m: Move) -> bool {
        let from = m.from();
        let to = m.to();

        let from_board = 1u64 << from;
        let to_board = 1u64 << to;
        let movement = from_board ^ to_board;

        let p = self.piece[from as usize];

        self.piece_bb[p.index() + S::OFFSET] ^= from_board;
        self.occupied ^= movement;

        if self.sq_attacked_by::<S::OPPOSITE>(self.get_king_square::<S>()) {
            self.piece_bb[p.index() + S::OFFSET] ^= from_board;
            self.occupied ^= movement;
            return false
        }

        self.color_bb[S::INDEX] ^= movement;
        self.piece[from as usize] = Empty;

        let promo = match m.flags() {
            Move::PROMO_QUEEN => Queen,
            Move::PROMO_KNIGHT => Knight,
            Move::PROMO_BISHOP => Bishop,
            Move::PROMO_ROOK => Rook,
            _ => panic!()
        };
        self.piece[to as usize] = promo;
        self.piece_bb[promo.index()] ^= to_board;

        self.en_passant = 0;
        self.turn = self.turn.opposite();
        true
    }

    #[inline(always)]
    fn move_piece<S: Side>(&mut self, m: Move) -> bool {
        let from = m.from();
        let to = m.to();
        let from_board = 1u64 << from;
        let to_board = 1u64 << to;
        let movement = from_board ^ to_board;
        let p = self.piece[from as usize];


        self.piece_bb[p.index() + S::OFFSET] ^= movement;
        self.occupied ^= movement;
        
        if self.sq_attacked_by::<S::OPPOSITE>(self.get_king_square::<S>()) {
            self.piece_bb[p.index() + S::OFFSET] ^= movement;
            self.occupied ^= movement;
            return false
        }

        self.color_bb[S::INDEX] ^= movement;

        self.piece[from as usize] = Empty;
        self.piece[to as usize] = p;

    
        self.castling_rights ^= CASTLING_RIGHTS[from as usize];    
        self.turn = self.turn.opposite();

        true
    }


    #[inline(always)]
    fn en_passant<S: Side>(&mut self, m: Move) -> bool {
        let from = m.from();
        let to = m.to();
        let from_board = 1u64 << from;
        let to_board = 1u64 << to;

        let movement = from_board ^ to_board;
        let p = self.piece[from as usize];

        self.piece_bb[p.index() + S::OFFSET] ^= movement;
        self.occupied ^= movement;
        
        if self.sq_attacked_by::<S::OPPOSITE>(self.get_king_square::<S>()) {
            self.piece_bb[p.index() + S::OFFSET] ^= movement;
            self.occupied ^= movement;
            return false
        }

        self.color_bb[S::INDEX] ^= movement;

        self.piece[from as usize] = Empty;
        self.piece[to as usize] = p;
        
        let captured_piece = EN_PASSANT_RM_SQUARES[to as usize];
        self.piece_bb[Pawn.index() + self.turn.opposite().offset()] ^= captured_piece;
        self.color_bb[S::OPPOSITE::INDEX] ^= captured_piece;
        
        self.en_passant = 0;
        self.turn = self.turn.opposite();
        true
    }


    #[inline(always)]
    fn capture<S: Side>(&mut self, m: Move) -> bool{
        let from = m.from();
        let to = m.to();
        let from_board = 1u64 << from;
        let to_board = 1u64 << to;

        let movement = from_board ^ to_board;
        
        let p_capturing = self.piece[from as usize];


        self.piece_bb[p_capturing.index() + S::OFFSET] ^= movement;
        self.occupied ^= from_board;

        if self.sq_attacked_by::<S::OPPOSITE>(self.get_king_square::<S>()) {
            self.piece_bb[p_capturing.index() + S::OFFSET] ^= movement;
            self.occupied ^= from_board;
            return false
        }
        
        let p_captured = self.piece[to as usize];

        // bitboard updates
        
        self.piece_bb[p_captured.index() + S::OPPOSITE::OFFSET] ^= to_board;
        
        self.color_bb[S::OPPOSITE::INDEX] ^= to_board;
        self.color_bb[S::INDEX] ^= movement;

        // 8x8 updates
        self.piece[from as usize] = Empty;
        self.piece[to as usize] = p_capturing;

        self.en_passant = 0;
        self.castling_rights ^= CASTLING_RIGHTS[from as usize];
        self.castling_rights ^= CASTLING_RIGHTS[to as usize];
        self.turn = self.turn.opposite();
        true
    }

    #[inline(always)]
    fn capture_promote<S: Side>(&mut self, m: Move) -> bool{
        let from = m.from();
        let to = m.to();
        let from_board = 1u64 << from;
        let to_board = 1u64 << to;

        let movement = from_board ^ to_board;
        
        let p_capturing = self.piece[from as usize];

        self.piece_bb[p_capturing.index() + S::OFFSET] ^= movement;
        self.occupied ^= from_board;

        if self.sq_attacked_by::<S::OPPOSITE>(self.get_king_square::<S>()) {
            self.piece_bb[p_capturing.index() + S::OFFSET] ^= movement;
            self.occupied ^= from_board;
            return false
        }
        
        let p_captured = self.piece[to as usize];

        // bitboard updates
        
        self.piece_bb[p_captured.index() + S::OPPOSITE::OFFSET] ^= to_board;
        self.color_bb[S::OPPOSITE::INDEX] ^= to_board;

        self.piece[from as usize] = Empty;
        self.piece[to as usize] = match m.flags() {
            Move::PROMO_QUEEN => Queen,
            Move::PROMO_KNIGHT => Knight,
            Move::PROMO_BISHOP => Bishop,
            Move::PROMO_ROOK => Rook,
            _ => panic!()
        };

        self.en_passant = 0;
        self.castling_rights ^= CASTLING_RIGHTS[to as usize];
        self.turn = self.turn.opposite();

        true
    }


    #[inline(always)]
    fn castle<S: Side>(&mut self, m: Move) {

        let mechs = &CASTLING_TABLE[S::INDEX][(m.flags() >> 2) as usize];
        self.castling_rights ^= mechs.castling_rights_update;
        self.piece_bb[S::OFFSET + King.index()] ^= mechs.king_movement;
        self.piece_bb[S::OFFSET + Rook.index()] ^= mechs.rook_movement;

        self.color_bb[self.turn.index()] ^= mechs.combined_movement;
        self.occupied ^= mechs.combined_movement;

        self.piece[mechs.king_disappears.index()] = Empty;
        self.piece[mechs.king_appears.index()] = King;

        self.piece[mechs.rook_disappears.index()] = Empty;
        self.piece[mechs.rook_appears.index()] = Rook;

        self.turn = self.turn.opposite();
    
    }

    #[inline(always)]
    pub fn make_move(&mut self, m: Move) -> bool {
        match self.turn {
            Color::White => self.make_pseudolegal_move::<WhiteSide>(m),
            Color::Black => self.make_pseudolegal_move::<BlackSide>(m),
            _ => panic!("Keiner am Zug!"),
        }
    }

    pub fn make_pseudolegal_move<S: Side>(&mut self, m: Move) -> bool{
        let mut success: bool = true;

        match m.flags() {
            Move::QUIET => {
                success = self.move_piece::<S>(m);
            },
            Move::CAPTURE => {
                success = self.capture::<S>(m);
            },
            Move::DOUBLE_PAWN => {
                success = self.move_piece::<S>(m);
                if success {
                    self.en_passant = EN_PESSANT_UPDATES[m.from() as usize];
                }
            },
            Move::EN_PASSANT => {
                success = self.en_passant::<S>(m);
            },

            _ => {
                if m.is_castle() {
                    self.castle::<S>(m);
                }
                else if m.is_simple_promo() {
                    success = self.promote::<S>(m);
                }
                else {
                    success = self.capture_promote::<S>(m);
                }
            }
        }

        success
    }




    pub fn get_color(&self, square: u8) -> Color {
        if self.piece_bb[0] & (1u64 << square) != 0 {
            return White
        }
        else {
            return Black
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
}
