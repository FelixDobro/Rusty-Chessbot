use super::Board;
use crate::chess::board::evaluation::MG;
use crate::chess::chess_move::*;
use crate::chess::constants::Color::{Black, White};
use crate::chess::constants::*;

use crate::chess::constants::Piece::*;
use crate::chess::board::UndoInfo;
use super::bitboard::EMPTY as EMPTY_BB;
use super::bitboard::*;
use crate::chess::square::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};
use std::f32::consts::E;

impl Board {
    pub fn generate_pseudolegals(&self) -> MoveList<MOVE_GEN_SIZE> {
        match self.turn {
            White => self.pseudolegal_moves::<WhiteSide>(),
            Black => self.pseudolegal_moves::<BlackSide>(),
            _ => panic!(),
        }
    }

    fn pseudolegal_moves<S: Side>(&self) -> MoveList<MOVE_GEN_SIZE> {
        let mut move_list = MoveList::new();
        self.pawn_moves::<S>(&mut move_list);
        self.knight_moves::<S>(&mut move_list);
        self.bishop_moves::<S>(&mut move_list);
        self.rook_moves::<S>(&mut move_list);
        self.queen_moves::<S>(&mut move_list);
        self.king_moves::<S>(&mut move_list);
        self.castling_moves::<S>(&mut move_list);
        move_list
    }

    #[inline(always)]
    pub fn castling_moves<S: Side>(&self, move_list: &mut MoveList<MOVE_GEN_SIZE>) {
        if !self.sq_attacked_by::<S::OPPOSITE>(Square::E1) {
            if self.castling_rights & CastlingRights::KingCastleWhite.index() != 0 {
                if self.occupied & WHITE_KING_CASTLE_BLOCKERS == EMPTY_BB {
                    if !(self.sq_attacked_by::<S::OPPOSITE>(Square::F1))
                        && !(self.sq_attacked_by::<S::OPPOSITE>(Square::G1))
                    {
                        move_list.push(Move::new(Square::E1, Square::G1, Move::KING_CASTLE));
                    }
                }
            }

            if self.castling_rights & CastlingRights::QueenCastleWhite.index() != 0 {
                if self.occupied & WHITE_QUEEN_CASTLE_BLOCKERS == EMPTY_BB {
                    if !(self.sq_attacked_by::<S::OPPOSITE>(Square::C1))
                        && !(self.sq_attacked_by::<S::OPPOSITE>(Square::D1))
                    {
                        move_list.push(Move::new(Square::E1, Square::C1, Move::QUEEN_CASTLE));
                    }
                }
            }
        }

        if !self.sq_attacked_by::<S::OPPOSITE>(Square::E8) {
            if self.castling_rights & CastlingRights::KingCastleBlack.index() != 0 {
                if self.occupied & BLACK_KING_CASTLE_BLOCKERS == EMPTY_BB {
                    if !(self.sq_attacked_by::<S::OPPOSITE>(Square::F8))
                        && !(self.sq_attacked_by::<S::OPPOSITE>(Square::G8))
                    {
                        move_list.push(Move::new(Square::E8, Square::G8, Move::KING_CASTLE));
                    }
                }
            }

            if self.castling_rights & CastlingRights::QueenCastleBlack.index() != 0 {
                if self.occupied & BLACK_QUEEN_CASTLE_BLOCKERS == EMPTY_BB {
                    if !(self.sq_attacked_by::<S::OPPOSITE>(Square::C8))
                        && !(self.sq_attacked_by::<S::OPPOSITE>(Square::D8))
                    {
                        move_list.push(Move::new(Square::E8, Square::C8, Move::QUEEN_CASTLE));
                    }
                }
            }
        }
    }

    #[inline(always)]
    pub fn king_moves<S: Side>(&self, move_list: &mut MoveList<MOVE_GEN_SIZE>) {
        let mut king_board = self.piece_bb[S::OFFSET + King.index()];

        while king_board != EMPTY_BB {
            let from_sqaure = king_board.lsb();

            let pattern_board = KING_PATTERNS[from_sqaure.usize()];
            let mut captures = pattern_board & self.color_bb[S::OPPOSITE::INDEX];
            let mut normal_moves =
                pattern_board & !self.color_bb[S::OPPOSITE::INDEX] & !self.color_bb[S::INDEX];

            while captures != EMPTY_BB {
                let to_square = captures.lsb();
                move_list.push(Move::new(from_sqaure, to_square, Move::CAPTURE));
                captures.pop_lsb();
            }

            while normal_moves != EMPTY_BB {
                let to_square = normal_moves.lsb();
                move_list.push(Move::new(from_sqaure, to_square, Move::QUIET));
                normal_moves.pop_lsb();
            }
            king_board.pop_lsb();
        }
    }

    #[inline(always)]
    pub fn queen_moves<S: Side>(&self, move_list: &mut MoveList<MOVE_GEN_SIZE>) {
        let mut queen_board = self.piece_bb[S::OFFSET + Queen.index()];

        while queen_board != EMPTY_BB {
            let from_sqaure = queen_board.lsb();

            let mask = STRAIGHT_LINES[from_sqaure.usize()];
            let index = unsafe { _pext_u64(self.occupied.u64(), mask.u64()) };
            let mut pattern_board = STRAIGHT_LINES_MAGIC[from_sqaure.usize()][index as usize];
            let mask = DIAGONAL_LINES[from_sqaure.usize()];
            let index = unsafe { _pext_u64(self.occupied.u64(), mask.u64()) };
            let diag_patterns = DIAG_LINES_MAGIC[from_sqaure.usize()][index as usize];

            pattern_board |= diag_patterns;

            let mut captures = pattern_board & self.color_bb[S::OPPOSITE::INDEX];
            let mut normal_moves =
                pattern_board & !self.color_bb[S::OPPOSITE::INDEX] & !self.color_bb[S::INDEX];

            while captures != EMPTY_BB {
                let to_square = captures.lsb();
                move_list.push(Move::new(from_sqaure, to_square, Move::CAPTURE));
                captures.pop_lsb();
            }

            while normal_moves != EMPTY_BB {
                let to_square = normal_moves.lsb();
                move_list.push(Move::new(from_sqaure, to_square, Move::QUIET));
                normal_moves.pop_lsb();
            }
            queen_board.pop_lsb();
        }
    }

    #[inline(always)]
    pub fn rook_moves<S: Side>(&self, move_list: &mut MoveList<MOVE_GEN_SIZE>) {
        let mut rook_board = self.piece_bb[S::OFFSET + Rook.index()];

        while rook_board != EMPTY_BB {
            let from_sqaure = rook_board.lsb();

            let mask = STRAIGHT_LINES[from_sqaure.usize()];
            let index = unsafe { _pext_u64(self.occupied.u64(), mask.u64()) };
            let pattern_board = STRAIGHT_LINES_MAGIC[from_sqaure.usize()][index as usize];

            let mut captures = pattern_board & self.color_bb[S::OPPOSITE::INDEX];
            let mut normal_moves =
                pattern_board & !self.color_bb[S::OPPOSITE::INDEX] & !self.color_bb[S::INDEX];

            while captures != EMPTY_BB {
                let to_square = captures.lsb();
                move_list.push(Move::new(from_sqaure, to_square, Move::CAPTURE));
                captures.pop_lsb();
            }

            while normal_moves != EMPTY_BB {
                let to_square = normal_moves.lsb();
                move_list.push(Move::new(from_sqaure, to_square, Move::QUIET));
                normal_moves.pop_lsb();
            }
            rook_board.pop_lsb();
        }
    }

    #[inline(always)]
    pub fn bishop_moves<S: Side>(&self, move_list: &mut MoveList<MOVE_GEN_SIZE>) {
        let mut bishop_board = self.piece_bb[S::OFFSET + Bishop.index()];

        while bishop_board != EMPTY_BB {
            let from_sqaure = bishop_board.lsb();

            let mask = DIAGONAL_LINES[from_sqaure.usize()];
            let index = unsafe { _pext_u64(self.occupied.u64(), mask.u64()) };
            let pattern_board = DIAG_LINES_MAGIC[from_sqaure.usize()][index as usize];

            let mut captures = pattern_board & self.color_bb[S::OPPOSITE::INDEX];
            let mut normal_moves =
                pattern_board & !self.color_bb[S::OPPOSITE::INDEX] & !self.color_bb[S::INDEX];

            while captures != EMPTY_BB {
                let to_square = captures.lsb();
                move_list.push(Move::new(from_sqaure, to_square, Move::CAPTURE));
                captures.pop_lsb();
            }

            while normal_moves != EMPTY_BB {
                let to_square = normal_moves.lsb();
                move_list.push(Move::new(from_sqaure, to_square, Move::QUIET));
                normal_moves.pop_lsb();
            }
            bishop_board.pop_lsb();
        }
    }

    #[inline(always)]
    pub fn knight_moves<S: Side>(&self, move_list: &mut MoveList<MOVE_GEN_SIZE>) {
        let mut knight_board = self.piece_bb[S::OFFSET + Knight.index()];

        while knight_board != EMPTY_BB {
            let from_sqaure = knight_board.lsb();

            let pattern_board = KNIGHT_PATTERNS[from_sqaure.usize()];
            let mut captures = pattern_board & self.color_bb[S::OPPOSITE::INDEX];
            let mut normal_moves =
                pattern_board & !self.color_bb[S::OPPOSITE::INDEX] & !self.color_bb[S::INDEX];

            while captures != EMPTY_BB {
                let to_square = captures.lsb();

                move_list.push(Move::new(from_sqaure, to_square, Move::CAPTURE));
                captures.pop_lsb();
            }

            while normal_moves != EMPTY_BB {
                let to_square = normal_moves.lsb();
                move_list.push(Move::new(from_sqaure, to_square, Move::QUIET));
                normal_moves.pop_lsb();
            }
            knight_board.pop_lsb();
        }
    }

    #[inline(always)]
    pub fn pawn_moves<S: Side>(&self, move_list: &mut MoveList<MOVE_GEN_SIZE>) {
        let pawn_board = self.piece_bb[Pawn.index() + S::OFFSET];

        let mut pushes = S::shift_up(pawn_board) & !self.occupied;
        let mut single_pushes = pushes & !S::LAST_RANK;

        let mut double_pushes = S::shift_up(single_pushes) & S::DOUBLE_PUSH_RANK & !self.occupied;
        let mut promotions = pushes & S::LAST_RANK;

        let attack_pattern_left = S::pawn_attack_pattern_l(pawn_board);
        let attack_pattern_right = S::pawn_attack_pattern_r(pawn_board);

        let all_left_attacks = attack_pattern_left & self.color_bb[S::OPPOSITE::INDEX];
        let mut left_attacks = all_left_attacks & !S::LAST_RANK;
        let mut left_promotions_cap = all_left_attacks & S::LAST_RANK;

        let all_right_attacks = attack_pattern_right & self.color_bb[S::OPPOSITE::INDEX];

        let mut right_attacks = all_right_attacks & !S::LAST_RANK;
        let mut right_promotions_cap = S::LAST_RANK & all_right_attacks;

        let mut left_en_passants = attack_pattern_left & self.en_passant;
        let mut right_en_passants = attack_pattern_right & self.en_passant;

        while single_pushes != EMPTY_BB {
            let to_square = single_pushes.lsb().i8();
            move_list.push(Move::new(
                Square::from_u16((to_square - S::UP) as u16),
                Square::from_u16(to_square as u16),
                Move::QUIET,
            ));
            single_pushes.pop_lsb();
        }

        while double_pushes != EMPTY_BB {
            let to_square = double_pushes.lsb().i8();
            move_list.push(Move::new(
                Square::from_u16((to_square - S::UP - S::UP) as u16),
                Square::from_u16(to_square as u16),
                Move::DOUBLE_PAWN,
            ));
            double_pushes.pop_lsb();
        }

        while promotions != EMPTY_BB {
            let to_square = Square::from_u16(promotions.lsb().u16());
            let from_square = Square::from_u16((to_square.i8() - S::UP) as u16);
            move_list.push(Move::new(from_square, to_square, Move::PROMO_KNIGHT));
            move_list.push(Move::new(from_square, to_square, Move::PROMO_QUEEN));
            move_list.push(Move::new(from_square, to_square, Move::PROMO_ROOK));
            move_list.push(Move::new(from_square, to_square, Move::PROMO_BISHOP));
            promotions.pop_lsb();
        }

        while left_attacks != EMPTY_BB {
            let to_square = left_attacks.lsb().i8();
            move_list.push(Move::new(
                Square::from_u16((to_square + S::DOWN_RIGHT) as u16),
                Square::from_u16(to_square as u16),
                Move::CAPTURE,
            ));
            left_attacks.pop_lsb();
        }

        while right_attacks != EMPTY_BB {
            let to_square = right_attacks.lsb().i8();
            move_list.push(Move::new(
                Square::from_u16((to_square + S::DOWN_LEFT) as u16),
                Square::from_u16(to_square as u16),
                Move::CAPTURE,
            ));
            right_attacks.pop_lsb();
        }

        while left_en_passants != EMPTY_BB {
            let to_square = left_en_passants.lsb().i8();
            move_list.push(Move::new(
                Square::from_u16((to_square + S::DOWN_RIGHT) as u16),
                Square::from_u16(to_square as u16),
                Move::EN_PASSANT,
            ));
            left_en_passants.pop_lsb();
        }

        while right_en_passants != EMPTY_BB {
            let to_square = right_en_passants.lsb().i8();
            move_list.push(Move::new(
                Square::from_u16((to_square + S::DOWN_LEFT) as u16),
                Square::from_u16(to_square as u16),
                Move::EN_PASSANT,
            ));
            right_en_passants.pop_lsb();
        }

        while left_promotions_cap != EMPTY_BB {
            let to_square_raw = left_promotions_cap.lsb().i8();
            let from_square = Square::from_u16((to_square_raw + S::DOWN_RIGHT) as u16);
            let to_square = Square::from_u16(to_square_raw as u16);
            move_list.push(Move::new(from_square, to_square, Move::PROMO_CAP_KNIGHT));
            move_list.push(Move::new(from_square, to_square, Move::PROMO_CAP_BISHOP));
            move_list.push(Move::new(from_square, to_square, Move::PROMO_CAP_QUEEN));
            move_list.push(Move::new(from_square, to_square, Move::PROMO_CAP_ROOK));
            left_promotions_cap.pop_lsb();
        }

        while right_promotions_cap != EMPTY_BB {
            let to_square_raw = right_promotions_cap.lsb().i8();
            let from_square = Square::from_u16((to_square_raw + S::DOWN_LEFT) as u16);
            let to_square = Square::from_u16(to_square_raw as u16);
            move_list.push(Move::new(from_square, to_square, Move::PROMO_CAP_KNIGHT));
            move_list.push(Move::new(from_square, to_square, Move::PROMO_CAP_BISHOP));
            move_list.push(Move::new(from_square, to_square, Move::PROMO_CAP_QUEEN));
            move_list.push(Move::new(from_square, to_square, Move::PROMO_CAP_ROOK));
            right_promotions_cap.pop_lsb();
        }
    }

    #[inline(always)]
    pub fn make_pl_move<const EVAL: bool>(&mut self, m: Move) -> bool {
        match self.turn {
            Color::White => self.make_pseudolegal_move::<WhiteSide, EVAL>(m),
            Color::Black => self.make_pseudolegal_move::<BlackSide, EVAL>(m),
            _ => panic!("No ones turn"),
        }
    }



    fn make_pseudolegal_move<S: Side, const EVAL: bool>(&mut self, m: Move) -> bool {
        let (from, to, from_board, to_board) = m.split();
        let movement = from_board ^ to_board;

        let piece_moved = self.piece[from.index()];
        let piece_captured = self.piece[to.index()];

        self.piece[from.index()] = Empty;
        self.piece[to.index()] = piece_moved;

        self.piece_bb[piece_moved.index() + S::OFFSET] ^= movement;
        self.color_bb[S::INDEX] ^= movement;

        let undo_info = UndoInfo {
            castling_rights: self.castling_rights,
            en_passant_square: self.en_passant,
            halfmove_clock: self.halfmoves,
            hash: self.hash,
            captured_piece: piece_captured,
            last_mg: self.eval_mg,
            last_eg: self.eval_eg,
            last_phase: self.game_phase
        };
        
        self.update_hash_caslte(self.castling_rights);
        self.update_hash_piece::<S>(piece_moved, from);
        self.update_hash_piece::<S>(piece_moved, to);
        self.halfmoves += 1;
        self.castling_rights &= !CASTLING_RIGHTS[from.usize()];
        
        if EVAL {
            self.rm_eval::<S>(piece_moved, from);
            self.add_eval::<S>(piece_moved, to);
        }

        if piece_captured != Empty {
            self.piece_bb[piece_captured.index() + S::OPPOSITE::OFFSET] ^= to_board;
            self.color_bb[S::OPPOSITE::INDEX] ^= to_board;
            self.update_hash_piece::<S::OPPOSITE>(piece_captured, to);
            self.halfmoves = 0;
            self.castling_rights &= !CASTLING_RIGHTS[to.usize()];
            if EVAL {
                self.rm_eval::<S::OPPOSITE>(piece_captured, to);
            }
        }

        let move_flags = m.flags();
        self.update_en_passant_hash();
        self.en_passant = EMPTY_BB;

        if move_flags == Move::DOUBLE_PAWN {
            self.en_passant = EN_PESSANT_UPDATES[from.index()];
            self.update_en_passant_hash();
            self.halfmoves = 0;
        }
        // Promotion
        else if move_flags > Move::EN_PASSANT {
            let promo = match move_flags {
                Move::PROMO_CAP_QUEEN | Move::PROMO_QUEEN => Queen,
                Move::PROMO_CAP_KNIGHT | Move::PROMO_KNIGHT=> Knight,
                Move::PROMO_CAP_BISHOP | Move::PROMO_BISHOP => Bishop,
                Move::PROMO_CAP_ROOK | Move::PROMO_ROOK => Rook,
                _ => panic!()
            };
            self.piece_bb[promo.index() + S::OFFSET] ^= to_board;
            self.piece_bb[Pawn.index() + S::OFFSET] ^= to_board;
            self.piece[to.index()] = promo;
            self.update_hash_piece::<S>(promo, to);
            self.update_hash_piece::<S>(Pawn, to);
            self.halfmoves = 0;
            if EVAL {
                self.rm_eval::<S>(piece_moved, to);
                self.add_eval::<S>(promo, to);
            }
        }
        else if move_flags == Move::EN_PASSANT {
            let pawn_remove_board = EN_PASSANT_RM_SQUARES[to.index()];
            self.piece_bb[Pawn.index() + S::OPPOSITE::OFFSET] ^= pawn_remove_board;
            self.color_bb[S::OPPOSITE::INDEX] ^= pawn_remove_board;
            let ep_square = pawn_remove_board.lsb();
            self.piece[ep_square.index()] = Empty;
            self.update_hash_piece::<S::OPPOSITE>(Pawn, ep_square);
            self.halfmoves = 0;
            if EVAL {
                self.rm_eval::<S>(Pawn, ep_square);
            }
        }
        else if m.is_castle() {
            let mechs = &CASTLING_TABLE[S::INDEX][(move_flags & 1) as usize];
            self.piece_bb[Rook.index() + S::OFFSET] ^= mechs.rook_movement;
            self.color_bb[S::INDEX] ^= mechs.rook_movement;
            self.piece[mechs.rook_appears.index()] = Rook;
            self.piece[mechs.rook_disappears.index()] = Empty;
            self.update_hash_piece::<S>(Rook, mechs.rook_appears);
            self.update_hash_piece::<S>(Rook, mechs.rook_disappears);
            if EVAL {
                self.rm_eval::<S>(Rook, mechs.rook_disappears);
                self.add_eval::<S>(Rook, mechs.rook_appears);
            }
        }   

        self.occupied = self.color_bb[0] | self.color_bb[1];
        self.turn = self.turn.opposite();
        self.update_move_hash();
        self.update_hash_caslte(self.castling_rights);
        if self.sq_attacked_by::<S::OPPOSITE>(self.get_king_square::<S>()) {
            self.unmake_pl_move_p::<S>(m, &undo_info);
            return false;
        }
        self.undo_stack.push(undo_info);
        true
    }

    fn unmake_pl_move_p<S: Side>(&mut self, m: Move, undo_info: &UndoInfo) {
        let (from, to, from_board, to_board) = m.split();
        let movement = from_board ^ to_board;

        let p_moved = self.piece[to.index()];
        
        self.piece_bb[p_moved.index() + S::OFFSET] ^= movement;
        self.color_bb[S::INDEX] ^= movement;
        self.piece[from.index()] = p_moved;
        self.piece[to.index()] = Empty;
        let captured_piece = undo_info.captured_piece;

        if captured_piece != Empty {
            self.piece_bb[captured_piece.index() + S::OPPOSITE::OFFSET] ^= to_board;
            self.color_bb[S::OPPOSITE::INDEX] ^= to_board;
            self.piece[to.index()] = captured_piece;
        }

        let move_flags = m.flags();

        if m.is_promo() {

            let promo = match move_flags {
                Move::PROMO_CAP_QUEEN | Move::PROMO_QUEEN => Queen,
                Move::PROMO_CAP_KNIGHT | Move::PROMO_KNIGHT=> Knight,
                Move::PROMO_CAP_BISHOP | Move::PROMO_BISHOP => Bishop,
                Move::PROMO_CAP_ROOK | Move::PROMO_ROOK => Rook,
                _ => panic!()
            };
            self.piece_bb[promo.index() + S::OFFSET] ^= from_board;
            self.piece_bb[S::OFFSET + Pawn.index()] ^= from_board;
            self.piece[from.index()] = Pawn
        }

        else if move_flags == Move::EN_PASSANT {
            let pawn_remove_board = EN_PASSANT_RM_SQUARES[to.index()];
            self.piece_bb[Pawn.index() + S::OPPOSITE::OFFSET] ^= pawn_remove_board;
            self.color_bb[S::OPPOSITE::INDEX] ^= pawn_remove_board;
            let ep_square = pawn_remove_board.lsb();
            self.piece[ep_square.index()] = Pawn;
            
        }
        else if m.is_castle() {
            let mechs = &CASTLING_TABLE[S::INDEX][(move_flags & 1) as usize];
            self.piece_bb[Rook.index() + S::OFFSET] ^= mechs.rook_movement;
            self.color_bb[S::INDEX] ^= mechs.rook_movement;
            self.piece[mechs.rook_appears.index()] = Empty;
            self.piece[mechs.rook_disappears.index()] = Rook;
        }
        self.occupied = self.color_bb[0] | self.color_bb[1];
        self.undo_state(undo_info);
    }


    #[inline(always)]
    fn undo_state(&mut self, undo_info: &UndoInfo) {
        self.castling_rights = undo_info.castling_rights;
        self.en_passant = undo_info.en_passant_square;
        self.halfmoves = undo_info.halfmove_clock;
        self.hash = undo_info.hash;
        self.turn = self.turn.opposite();
        self.eval_eg = undo_info.last_eg;
        self.eval_mg = undo_info.last_mg;
        self.game_phase = undo_info.last_phase;
        
    }

  


    pub fn unmake_pl_move(&mut self, m: Move) {
        let undo_info = self.undo_stack.pop();
        match self.turn {
            White => self.unmake_pl_move_p::<BlackSide>(m, &undo_info),
            Black => self.unmake_pl_move_p::<WhiteSide>(m, &undo_info),
            _ => panic!("No ones turn?"),
        }
    }
}
