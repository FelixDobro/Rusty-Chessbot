
use crate::chess::board::Board;
use crate::chess::chessMove::{*};
use crate::chess::constants::{*};
use crate::chess::constants::Color::{White, Black};

use crate::chess::constants::Piece::{*};

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};


impl Board {

    pub fn generate_pseudolegals(&self) -> MoveList {
        match self.turn {
            White => self.pseudolegal_moves::<WhiteSide>(),
            Black => self.pseudolegal_moves::<BlackSide>(),
            _ => panic!(),
        }
    }

    fn pseudolegal_moves<S: Side>(&self) -> MoveList {
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
    pub fn castling_moves<S: Side>(&self, move_list: &mut MoveList) {
        
        
        
        if !self.sq_attacked_by::<S::OPPOSITE>(Square::E1) {
            if self.castling_rights & CASTLING_RIGHTS::KingCastleWhite.index() != 0 {
            if self.occupied & WHITE_KING_CASTLE_BLOCKERS == 0 {
                if !(self.sq_attacked_by::<S::OPPOSITE>(Square::F1)) 
                && !(self.sq_attacked_by::<S::OPPOSITE>(Square::G1))
                {
                    move_list.push(Move::new(Square::E1 as u16, Square::G1 as u16, Move::KING_CASTLE));
                }
                }
            }

            if self.castling_rights & CASTLING_RIGHTS::QueenCastleWhite.index() != 0 {
            if self.occupied & WHITE_QUEEN_CASTLE_BLOCKERS == 0 {
                if !(self.sq_attacked_by::<S::OPPOSITE>(Square::C1))
                && !(self.sq_attacked_by::<S::OPPOSITE>(Square::D1)) {
                    move_list.push(Move::new(Square::E1 as u16, Square::C1 as u16, Move::QUEEN_CASTLE));
                }
                }
            }
        }
        

        if !self.sq_attacked_by::<S::OPPOSITE>(Square::E8) {
            if self.castling_rights & CASTLING_RIGHTS::KingCastleBlack.index() != 0 {
            if self.occupied & BLACK_KING_CASTLE_BLOCKERS == 0 {
                if !(self.sq_attacked_by::<S::OPPOSITE>(Square::F8)) 
                && !(self.sq_attacked_by::<S::OPPOSITE>(Square::G8)) {
                    move_list.push(Move::new(Square::E8 as u16, Square::G8 as u16, Move::KING_CASTLE));
                    }
                }
        }

        

        if self.castling_rights & CASTLING_RIGHTS::QueenCastleBlack.index() != 0 {
            if self.occupied & BLACK_QUEEN_CASTLE_BLOCKERS == 0 {
                if !(self.sq_attacked_by::<S::OPPOSITE>(Square::C8))
                && !(self.sq_attacked_by::<S::OPPOSITE>(Square::D8)) {
                    move_list.push(Move::new(Square::E8 as u16, Square::C8 as u16, Move::QUEEN_CASTLE));
                }
                }
            }
        }
        
    }

    #[inline(always)]
    pub fn king_moves<S: Side>(&self, move_list: &mut MoveList) {
        let mut knight_board = self.piece_bb[S::OFFSET + King.index()];

        while knight_board != 0 {
            let from_sqaure = knight_board.trailing_zeros() as u16;

            let pattern_board = KING_PATTERNS[from_sqaure as usize];
            let mut captures = pattern_board & self.color_bb[S::OPPOSITE::INDEX];
            let mut normal_moves =
                pattern_board & !self.color_bb[S::OPPOSITE::INDEX] & !self.color_bb[S::INDEX];

            while captures != 0 {
                let to_square = captures.trailing_zeros() as u16;
                move_list.push(Move::new(from_sqaure, to_square, Move::CAPTURE));
                captures &= captures - 1;
            }

            while normal_moves != 0 {
                let to_square = normal_moves.trailing_zeros() as u16;
                move_list.push(Move::new(from_sqaure, to_square, Move::QUIET));
                normal_moves &= normal_moves - 1;
            }
            knight_board &= knight_board - 1;
        }
    }


    #[inline(always)]
    pub fn queen_moves<S: Side>(&self, move_list: &mut MoveList) {
        let mut queen_board = self.piece_bb[S::OFFSET + Queen.index()];

        while queen_board != 0 {
            let from_sqaure = queen_board.trailing_zeros() as u16;

            let mask = STRAIGHT_LINES[from_sqaure as usize];
            let index = unsafe { _pext_u64(self.occupied, mask) };
            let mut pattern_board = STRAIGHT_LINES_MAGIC[from_sqaure as usize][index as usize];
            let mask = DIAGONAL_LINES[from_sqaure as usize];
            let index = unsafe { _pext_u64(self.occupied, mask)};
            let diag_patterns = DIAG_LINES_MAGIC[from_sqaure as usize][index as usize];

            pattern_board |= diag_patterns;


            let mut captures = pattern_board & self.color_bb[S::OPPOSITE::INDEX];
            let mut normal_moves =
                pattern_board & !self.color_bb[S::OPPOSITE::INDEX] & !self.color_bb[S::INDEX];

            while captures != 0 {
                let to_square = captures.trailing_zeros() as u16;
                move_list.push(Move::new(from_sqaure, to_square, Move::CAPTURE));
                captures &= captures - 1;
            }

            while normal_moves != 0 {
                let to_square = normal_moves.trailing_zeros() as u16;
                move_list.push(Move::new(from_sqaure, to_square, Move::QUIET));
                normal_moves &= normal_moves - 1;
            }
            queen_board &= queen_board - 1;
        }
    }

    #[inline(always)]
    pub fn rook_moves<S: Side>(&self, move_list: &mut MoveList) {
        let mut rook_board = self.piece_bb[S::OFFSET + Rook.index()];

        while rook_board != 0 {
            let from_sqaure = rook_board.trailing_zeros() as u16;

            let mask = STRAIGHT_LINES[from_sqaure as usize];
            let index = unsafe { _pext_u64(self.occupied, mask) };
            let pattern_board = STRAIGHT_LINES_MAGIC[from_sqaure as usize][index as usize];

            let mut captures = pattern_board & self.color_bb[S::OPPOSITE::INDEX];
            let mut normal_moves =
                pattern_board & !self.color_bb[S::OPPOSITE::INDEX] & !self.color_bb[S::INDEX];

            while captures != 0 {
                let to_square = captures.trailing_zeros() as u16;
                move_list.push(Move::new(from_sqaure, to_square, Move::CAPTURE));
                captures &= captures - 1;
            }

            while normal_moves != 0 {
                let to_square = normal_moves.trailing_zeros() as u16;
                move_list.push(Move::new(from_sqaure, to_square, Move::QUIET));
                normal_moves &= normal_moves - 1;
            }
            rook_board &= rook_board - 1;
        }
    }


    #[inline(always)]
    pub fn bishop_moves<S: Side>(&self, move_list: &mut MoveList) {
        let mut bishop_board = self.piece_bb[S::OFFSET + Bishop.index()];

        while bishop_board != 0 {
            let from_sqaure = bishop_board.trailing_zeros() as u16;

            let mask = DIAGONAL_LINES[from_sqaure as usize];
            let index = unsafe { _pext_u64(self.occupied, mask) };
            let pattern_board = DIAG_LINES_MAGIC[from_sqaure as usize][index as usize];

            let mut captures = pattern_board & self.color_bb[S::OPPOSITE::INDEX];
            let mut normal_moves =
                pattern_board & !self.color_bb[S::OPPOSITE::INDEX] & !self.color_bb[S::INDEX];

            while captures != 0 {
                let to_square = captures.trailing_zeros() as u16;
                move_list.push(Move::new(from_sqaure, to_square, Move::CAPTURE));
                captures &= captures - 1;
            }

            while normal_moves != 0 {
                let to_square = normal_moves.trailing_zeros() as u16;
                move_list.push(Move::new(from_sqaure, to_square, Move::QUIET));
                normal_moves &= normal_moves - 1;
            }
            bishop_board &= bishop_board - 1;
        }
    }

    #[inline(always)]
    pub fn knight_moves<S: Side>(&self, move_list: &mut MoveList) {
        let mut knight_board = self.piece_bb[S::OFFSET + Knight.index()];

        while knight_board != 0 {
            let from_sqaure = knight_board.trailing_zeros() as u16;

            let pattern_board = KNIGHT_PATTERNS[from_sqaure as usize];
            let mut captures = pattern_board & self.color_bb[S::OPPOSITE::INDEX];
            let mut normal_moves =
                pattern_board & !self.color_bb[S::OPPOSITE::INDEX] & !self.color_bb[S::INDEX];

            while captures != 0 {
                let to_square = captures.trailing_zeros() as u16;
                move_list.push(Move::new(from_sqaure, to_square, Move::CAPTURE));
                captures &= captures - 1;
            }

            while normal_moves != 0 {
                let to_square = normal_moves.trailing_zeros() as u16;
                move_list.push(Move::new(from_sqaure, to_square, Move::QUIET));
                normal_moves &= normal_moves - 1;
            }
            knight_board &= knight_board - 1;
        }
    }

    #[inline(always)]
    pub fn pawn_moves<S: Side>(&self, move_list: &mut MoveList) {

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

        while single_pushes != 0 {
            let to_square = single_pushes.trailing_zeros() as i8;
            move_list.push(Move::new((to_square - S::UP) as u16, to_square as u16, 0));
            single_pushes &= single_pushes - 1;
        }

        while double_pushes != 0 {
            let to_square = double_pushes.trailing_zeros() as i8;
            move_list.push(Move::new(
                (to_square - S::UP - S::UP) as u16,
                to_square as u16,
                1,
            ));
            double_pushes &= double_pushes - 1;
        }

        while promotions != 0 {
            let to_square = promotions.trailing_zeros() as i8;
            let from_square = (to_square - S::UP) as u16;
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_KNIGHT));
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_QUEEN));
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_ROOK));
            move_list.push(Move::new(from_square, to_square as u16, Move::PROMO_BISHOP));
            promotions &= promotions - 1;
        }

        while left_attacks != 0 {
            let to_square = left_attacks.trailing_zeros() as i8;
            move_list.push(Move::new(
                (to_square + S::DOWN_RIGHT) as u16,
                to_square as u16,
                Move::CAPTURE,
            ));
            left_attacks &= left_attacks - 1;
        }

        while right_attacks != 0 {
            let to_square = right_attacks.trailing_zeros() as i8;
            move_list.push(Move::new(
                (to_square + S::DOWN_LEFT) as u16,
                to_square as u16,
                Move::CAPTURE,
            ));
            right_attacks &= right_attacks - 1;
        }

        while left_en_passants != 0 {
            let to_square = left_en_passants.trailing_zeros() as i8;
            move_list.push(Move::new(
                (to_square + S::DOWN_RIGHT) as u16,
                to_square as u16,
                Move::EN_PASSANT,
            ));
            left_en_passants &= left_en_passants - 1;
        }

        while right_en_passants != 0 {

            let to_square = right_en_passants.trailing_zeros() as i8;
            move_list.push(Move::new(
                (to_square + S::DOWN_LEFT) as u16,
                to_square as u16,
                Move::EN_PASSANT,
            ));
            right_en_passants &= right_en_passants - 1;
        }

        while left_promotions_cap != 0 {
            let to_square = left_promotions_cap.trailing_zeros() as i8;
            let from_square = (to_square + S::DOWN_RIGHT) as u16;
            move_list.push(Move::new(
                from_square,
                to_square as u16,
                Move::PROMO_CAP_KNIGHT,
            ));
            move_list.push(Move::new(
                from_square,
                to_square as u16,
                Move::PROMO_CAP_BISHOP,
            ));
            move_list.push(Move::new(
                from_square,
                to_square as u16,
                Move::PROMO_CAP_QUEEN,
            ));
            move_list.push(Move::new(
                from_square,
                to_square as u16,
                Move::PROMO_CAP_ROOK,
            ));
            left_promotions_cap &= left_promotions_cap - 1;
        }

        while right_promotions_cap != 0 {
            let to_square = right_promotions_cap.trailing_zeros() as i8;
            let from_square = (to_square + S::DOWN_LEFT) as u16;
            move_list.push(Move::new(
                from_square,
                to_square as u16,
                Move::PROMO_CAP_KNIGHT,
            ));
            move_list.push(Move::new(
                from_square,
                to_square as u16,
                Move::PROMO_CAP_BISHOP,
            ));
            move_list.push(Move::new(
                from_square,
                to_square as u16,
                Move::PROMO_CAP_QUEEN,
            ));
            move_list.push(Move::new(
                from_square,
                to_square as u16,
                Move::PROMO_CAP_ROOK,
            ));
            right_promotions_cap &= right_promotions_cap - 1;
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
            return false;
        }

        self.color_bb[S::INDEX] ^= movement;
        self.piece[from as usize] = Empty;

        let promo = match m.flags() {
            Move::PROMO_QUEEN => Queen,
            Move::PROMO_KNIGHT => Knight,
            Move::PROMO_BISHOP => Bishop,
            Move::PROMO_ROOK => Rook,
            _ => panic!(),
        };
        self.piece[to as usize] = promo;
        self.piece_bb[promo.index() + S::OFFSET] ^= to_board;

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
            return false;
        }

        self.color_bb[S::INDEX] ^= movement;

        self.piece[from as usize] = Empty;
        self.piece[to as usize] = p;

        self.castling_rights &= !CASTLING_RIGHTS[from as usize];
        self.turn = self.turn.opposite();
        self.en_passant = 0;
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


        let remove_board = EN_PASSANT_RM_SQUARES[to as usize];
        self.occupied ^= remove_board | remove_board;
        self.piece_bb[p.index() + S::OPPOSITE::OFFSET] ^= remove_board;
        if self.sq_attacked_by::<S::OPPOSITE>(self.get_king_square::<S>()) {
            self.piece_bb[p.index() + S::OFFSET] ^= movement;
            self.occupied ^= movement | remove_board;
            self.piece_bb[p.index() + S::OPPOSITE::OFFSET] ^= remove_board;
            return false;
        }

        self.color_bb[S::OPPOSITE::INDEX] ^= remove_board;
        
        self.color_bb[S::INDEX] ^= movement;

        self.piece[from as usize] = Empty;
        self.piece[to as usize] = p;
        self.piece[remove_board.trailing_zeros() as usize] = Empty;

        self.en_passant = 0;
        self.turn = self.turn.opposite();
        true
    }

    #[inline(always)]
    fn capture<S: Side>(&mut self, m: Move) -> bool {
        let from = m.from();
        let to = m.to();
        let from_board = 1u64 << from;
        let to_board = 1u64 << to;

        let movement = from_board ^ to_board;

        let p_capturing = self.piece[from as usize];
        let p_captured = self.piece[to as usize];

        self.piece_bb[p_capturing.index() + S::OFFSET] ^= movement;
        self.occupied ^= from_board;
        
        self.piece_bb[p_captured.index() + S::OPPOSITE::OFFSET] ^= to_board;

        if self.sq_attacked_by::<S::OPPOSITE>(self.get_king_square::<S>()) {
            self.piece_bb[p_capturing.index() + S::OFFSET] ^= movement;
            self.occupied ^= from_board;
            self.piece_bb[p_captured.index() + S::OPPOSITE::OFFSET] ^= to_board;
            return false;
        }


        self.color_bb[S::OPPOSITE::INDEX] ^= to_board;
        self.color_bb[S::INDEX] ^= movement;

        // 8x8 updates
        self.piece[from as usize] = Empty;
        self.piece[to as usize] = p_capturing;

        self.en_passant = 0;
        self.castling_rights &= !CASTLING_RIGHTS[from as usize];
        self.castling_rights &= !CASTLING_RIGHTS[to as usize];
        self.turn = self.turn.opposite();
        true
    }

    #[inline(always)]
    fn capture_promote<S: Side>(&mut self, m: Move) -> bool {
        let from = m.from();
        let to = m.to();
        let from_board = 1u64 << from;
        let to_board = 1u64 << to;

        let movement = from_board ^ to_board;

        let p_capturing = self.piece[from as usize];

        self.piece_bb[p_capturing.index() + S::OFFSET] ^= movement;
        self.occupied ^= from_board;

        let p_captured = self.piece[to as usize];
        
        self.piece_bb[p_captured.index() + S::OPPOSITE::OFFSET] ^= to_board;

        if self.sq_attacked_by::<S::OPPOSITE>(self.get_king_square::<S>()) {
            self.piece_bb[p_capturing.index() + S::OFFSET] ^= movement;
            self.occupied ^= from_board;
            self.piece_bb[p_captured.index() + S::OPPOSITE::OFFSET] ^= to_board;
            return false;
        }

        // bitboard updates
        
        self.color_bb[S::OPPOSITE::INDEX] ^= to_board;
        self.color_bb[S::INDEX] ^= movement;

        self.piece[from as usize] = Empty;
        self.piece[to as usize] = match m.flags() {
            Move::PROMO_CAP_QUEEN=> Queen,
            Move::PROMO_CAP_KNIGHT => Knight,
            Move::PROMO_CAP_BISHOP => Bishop,
            Move::PROMO_CAP_ROOK => Rook,
            _ => panic!(),
        };


        self.en_passant = 0;
        self.castling_rights &= !CASTLING_RIGHTS[to as usize];
        self.turn = self.turn.opposite();

        true
    }

    #[inline(always)]
    fn castle<S: Side>(&mut self, m: Move) {
        let mechs = &CASTLING_TABLE[S::INDEX][(m.flags() & 1) as usize];
        self.castling_rights &= !mechs.castling_rights_update;

        self.piece_bb[S::OFFSET + King.index()] ^= mechs.king_movement;
        self.piece_bb[S::OFFSET + Rook.index()] ^= mechs.rook_movement;

        self.color_bb[self.turn.index()] ^= mechs.combined_movement;

        self.occupied ^= mechs.combined_movement;

        self.piece[mechs.king_disappears.index()] = Empty;
        self.piece[mechs.king_appears.index()] = King;

        self.piece[mechs.rook_disappears.index()] = Empty;
        self.piece[mechs.rook_appears.index()] = Rook;
        self.en_passant = 0;
        self.turn = self.turn.opposite();
    }

    #[inline(always)]
    pub fn make_pl_move(&mut self, m: Move) -> bool {
        match self.turn {
            Color::White => self.make_pseudolegal_move::<WhiteSide>(m),
            Color::Black => self.make_pseudolegal_move::<BlackSide>(m),
            _ => panic!("No ones turn"),
        }
    }

    #[inline(always)]
    pub fn make_pl_move_copy(&self, m: Move) -> Option<Board> {
        let mut board = self.clone();
        let success = match self.turn {
            Color::White => board.make_pseudolegal_move::<WhiteSide>(m),
            Color::Black => board.make_pseudolegal_move::<BlackSide>(m),
            _ => panic!("No ones turn"),
        };
        if success {return Some(board)}
        return None
    }

    fn unmake_pseudolegal_move<S: Side>(&mut self, m: Move) {
        match m.flags() {
            Move::QUIET => {
                self.move_piece::<S>(m);
            },
            Move::CAPTURE => {},
            _ => {}
        }
    }

    fn make_pseudolegal_move<S: Side>(&mut self, m: Move) -> bool {
        let mut success: bool = true;

        match m.flags() {
            Move::QUIET => {
                success = self.move_piece::<S>(m);
            }
            Move::CAPTURE => {
                success = self.capture::<S>(m);
            }
            Move::DOUBLE_PAWN => {
                success = self.move_piece::<S>(m);
                if success {
                    self.en_passant = EN_PESSANT_UPDATES[m.from() as usize];
                }
            }
            Move::EN_PASSANT => {
                success = self.en_passant::<S>(m);
            }

            _ => {
                if m.is_castle() {
                    self.castle::<S>(m);
                } else if m.is_simple_promo() {
                    success = self.promote::<S>(m);
                } else {
                    success = self.capture_promote::<S>(m);
                }
            }
        }
        if success {
            self.halfmoves += 1;
            self.fullmoves += S::INDEX as u16;
        }
        success
    }


}