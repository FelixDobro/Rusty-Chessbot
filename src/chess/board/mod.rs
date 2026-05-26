pub mod bitboard;
pub mod hash;
pub mod move_gen;
pub mod evaluation;


use bitboard::EMPTY as EMPTY_BB;
use bitboard::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{_pdep_u64, _pext_u64};
use std::error::Error;
use std::{default, str};

use crate::chess::chess_move::{GAME_MOVES_SIZE, Move, MoveList};
use hash::HashList;
use crate::chess::square::Square;

use crate::chess::constants::{*};
use crate::chess::constants::Color::{Black, White};
use crate::chess::constants::Piece::*;
use crate::chess::constants::{BlackSide, CastlingRights, Color, Piece, Side, WhiteSide};


pub const GAME_POSITIONS_SIZE:usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UndoInfo {
    pub castling_rights: u8,
    pub en_passant_square: Bitboard, 
    pub halfmove_clock: u16,
    pub hash: u64,
    pub captured_piece: Piece,
    pub last_mg: i16,
    pub last_eg: i16,
    pub last_phase: i16,
}

impl UndoInfo {
    
    pub fn empty() -> Self {
        Self { castling_rights: 0, en_passant_square: EMPTY_BB, halfmove_clock: 0, captured_piece: Empty, hash: 0, last_eg: 0, last_mg: 0, last_phase: 0}
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoStack {
    undo_stack: [UndoInfo; GAME_POSITIONS_SIZE],
    count: usize
}

impl UndoStack {

    pub fn new() -> Self {
        UndoStack { undo_stack: [UndoInfo::empty(); GAME_POSITIONS_SIZE], count: 0}
    }

    #[inline(always)]
    pub fn push(&mut self, info: UndoInfo) {
        self.undo_stack[self.count] = info;
        self.count += 1;
    }

    #[inline(always)]
    pub fn pop(&mut self) -> UndoInfo {
        self.count -= 1;
        self.undo_stack[self.count]
    }
}



#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(align(64))]

pub struct Board {
    piece_bb: [Bitboard; 12],
    color_bb: [Bitboard; 2],
    occupied: Bitboard,

    piece: [Piece; 64],

    turn: Color,
    en_passant: Bitboard,
    castling_rights: u8,
    
    eval_mg: i16,
    eval_eg: i16,
    game_phase: i16,
    hash: u64,
    halfmoves: u16,
    fullmove_counter: u16,

    undo_stack: Box<UndoStack>,
    positions: HashList<GAME_POSITIONS_SIZE>,
    
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
            hash: 0,
            fullmove_counter: 1,
            positions: HashList::new(),
            undo_stack: Box::new(UndoStack::new()),
            eval_eg: 0,
            eval_mg: 0,
            game_phase: 0,
        };
        board.hash = board.calculate_hash();
        board.game_phase = board.calc_phase();
        board.eval_mg = board.calculate_mg();
        board.eval_eg = board.calculate_eg();
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
        let mut halfmoves_b: u16 = 1;
        let mut fullmoves_b = 0;

        let mut splitted = fen_string.split(" ");

        if let Some(ranks) = splitted.next() {
            let mut num_ranks = 0;

            if num_ranks > 7 {
                return Err(FenError::InvalidNumRanks);
            }

            for rank in ranks.split("/") {
                let mut square_offset: u8 = (7 - num_ranks) * 8;
                for c in rank.chars() {
                    if let Some(number) = c.to_digit(10) {
                        square_offset += number as u8;
                    } else {
                        let color = if c.is_uppercase() { White } else { Black };

                        let piece = match c.to_ascii_lowercase() {
                            'p' => Pawn,
                            'n' => Knight,
                            'b' => Bishop,
                            'r' => Rook,
                            'q' => Queen,
                            'k' => King,
                            _ => return Err(FenError::InvalidPiece),
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
            if num_ranks != 8 {
                return Err(FenError::InvalidNumRanks);
            }
        } else {
            return Err(FenError::InvalidNumSections);
        }

        if let Some(side_to_move) = splitted.next() {
            turn = match side_to_move {
                "w" => White,
                "b" => Black,
                _ => return Err(FenError::InvalidTurn),
            };
        } else {
            return Err(FenError::InvalidNumSections);
        }

        if let Some(rights) = splitted.next() {
            for right in rights.chars() {
                castling_rights += match right {
                    'K' => CastlingRights::KingCastleWhite as u8,
                    'Q' => CastlingRights::QueenCastleWhite as u8,
                    'k' => CastlingRights::KingCastleBlack as u8,
                    'q' => CastlingRights::QueenCastleBlack as u8,
                    '-' => 0,
                    _ => return Err(FenError::Castling),
                }
            }
        } else {
            return Err(FenError::InvalidNumSections);
        }

        if let Some(en_passant) = splitted.next() {
            en_passant_right = match en_passant {
                "-" => EMPTY_BB,
                _ => {
                    if let Ok(square) = Square::from_string(en_passant) {
                        square.to_bitboard()
                    } else {
                        return Err(FenError::EnPassant);
                    }
                }
            };
        }

        if let Some(halfmoves) = splitted.next() {
            if let Ok(num) = halfmoves.parse::<u16>() {
                halfmoves_b = num
            } else {
                return Err(FenError::HalfMove);
            }
        }

        if let Some(fullmove) = splitted.next() {
            if let Ok(num) = fullmove.parse::<u16>() {
                fullmoves_b = num;
            } else {
                return Err(FenError::FullMove);
            }
        }

        let mut board = Board {
            piece_bb,
            color_bb,
            occupied,
            piece: piece_8_board,
            turn,
            en_passant: en_passant_right,
            castling_rights,
            halfmoves: halfmoves_b,
            hash: 0,
            eval_eg: 0,
            eval_mg: 0,
            game_phase: 0,
            fullmove_counter: fullmoves_b,
            positions: HashList::new(),
            undo_stack: Box::new(UndoStack::new())
        };
        board.hash = board.calculate_hash();
        board.eval_eg = board.calculate_eg();
        board.eval_mg = board.calculate_mg();
        board.game_phase = board.calc_phase();
        Ok(board)
    }

    #[inline(always)]
    pub fn get_hash(&self) -> u64 {
        self.hash
    }

   
    #[inline(always)]
    pub fn get_turn(&self) -> Color {
        self.turn
    }

    #[inline(always)]
    pub fn get_halfmoves(&self) -> u16 {
        self.halfmoves
    }

    #[inline(always)]
    pub fn get_enpassant(&self) -> Bitboard {
        self.en_passant
    }

    #[inline(always)]
    pub fn get_pieces(&self) -> [Piece; 64] {
        self.piece
    }

    #[inline(always)]
    pub fn get_mg(&self) -> i16 {
        self.eval_mg
    }

    #[inline(always)]
    pub fn get_eg(&self) -> i16 {
        self.eval_eg
    }

    #[inline(always)]
    pub fn get_phase(&self) -> i16 {
        self.game_phase
    }

    #[inline(always)]
    pub fn get_castling_rights(&self) -> u8 {
        self.castling_rights
    }

    #[inline(always)]
    pub fn get_all_bitboards(&self) -> [Bitboard; 12] {
        self.piece_bb
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

    pub fn count_material(&self) -> i16 {
        let mut result = 0i16;
        result += self.piece_bb[Pawn.index()].count_ones() as i16;
        result += self.piece_bb[Bishop.index()].count_ones() as i16 * 3;
        result += self.piece_bb[Knight.index()].count_ones() as i16 * 3;
        result += self.piece_bb[Rook.index()].count_ones() as i16 * 5;
        result += self.piece_bb[Queen.index()].count_ones() as i16 * 9;

        result -= self.piece_bb[Pawn.index() + Black.offset()].count_ones() as i16;
        result -= self.piece_bb[Bishop.index() + Black.offset()].count_ones() as i16 * 3;
        result -= self.piece_bb[Knight.index() + Black.offset()].count_ones() as i16 * 3;
        result -= self.piece_bb[Rook.index() + Black.offset()].count_ones() as i16 * 5;
        result -= self.piece_bb[Queen.index() + Black.offset()].count_ones() as i16 * 9;

        match self.turn {
            White => result,
            _ => -result,
        }
    }

    

    pub fn qualify_move(&self, m: &str) -> Result<Move, Box<dyn Error>> {
        match self.turn {
            White => self.move_matching::<WhiteSide>(m),
            Black => self.move_matching::<BlackSide>(m),
            _ => panic!(),
        }
    }

    #[inline(always)]
    fn move_matching<S: Side>(&self, m: &str) -> Result<Move, Box<dyn Error>> {
        let first = &m[0..2];
        let second = &m[2..4];
        let from_square = Square::from_string(first)?;
        let to_square = Square::from_string(second)?;
        let from_piece = self.get_piece(from_square);
        let to_piece = self.get_piece(to_square);
        let to_board = to_square.to_bitboard();
        let mut flags = Move::QUIET;

        match from_piece {
            King => {
                if from_square == Square::E1 || from_square == Square::E8 {
                    if to_square == Square::G1 || to_square == Square::G8 {
                        flags = Move::KING_CASTLE;
                    } else if to_square == Square::C1 || to_square == Square::C8 {
                        flags = Move::QUEEN_CASTLE;
                    }
                }
                if to_piece != Empty {
                    flags = Move::CAPTURE
                }
            }
            Pawn => match to_piece {
                Empty => {
                    if to_board == self.en_passant {
                        flags = Move::EN_PASSANT;
                    } else if S::shift_up(from_square.to_bitboard()) & to_board != EMPTY_BB {
                        if to_board & S::LAST_RANK != EMPTY_BB {
                            if let Some(c) = m.chars().nth(4) {
                            
                                flags = match c {
                                'q' => Move::PROMO_QUEEN,
                                'n' => Move::PROMO_KNIGHT,
                                'r' => Move::PROMO_ROOK,
                                'b' => Move::PROMO_BISHOP,
                                _ => panic!(),
                            };
                            } else {
                                return Err(
                                    format!("Should be en passant_capture, no 5th char").into()
                                );
                            }
                        }
                    } else {
                        flags = Move::DOUBLE_PAWN;
                    }
                }
                _ => {
                   
              
                    if to_board & S::LAST_RANK != EMPTY_BB {
                        
                        if let Some(c) = m.chars().nth(4) {
                            flags = match c {
                                'q' => Move::PROMO_CAP_QUEEN,
                                'n' => Move::PROMO_CAP_KNIGHT,
                                'r' => Move::PROMO_CAP_ROOK,
                                'b' => Move::PROMO_CAP_BISHOP,
                                _ => panic!(),
                            };
                        } else {
                            return Err(format!("Should be en passant_capture, no 5th char").into());
                        }
                    } else {
                   
                        flags = Move::CAPTURE;
                    }
                }
            },
            _ => {
                if to_piece != Empty {
                    flags = Move::CAPTURE
                }
            }
        }
        Ok(Move::new(from_square, to_square, flags))
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


    pub fn can_claim_draw(&self) -> bool {
        let halfmoves = self.halfmoves as u64;

        if halfmoves > 99 {
            return true;
        }

        let mut num_occurences = 0;
        let current_hash = self.hash;

        for &hash in self.positions.half_move_iter(halfmoves) {
            if current_hash == hash {
                num_occurences += 1
            }
        }

        num_occurences > 2
    }


}


#[cfg(test)]
mod test {

    use crate::chess::chess_move::Move;
    use crate::chess::square::Square;
    use crate::chess::board::Board;


    #[test]
    fn default_should_be_equal_0() {
        assert_eq!(
            Board::default().count_material(),
            0,
            "Default evaluation is non Zero"
        );
    }

    #[test]
    fn black_loses_pawn() {
        let m1 = Move::new(Square::E2, Square::E4, 1);
        let m2 = Move::new(Square::B7, Square::B5, 1);
        let m3 = Move::new(Square::F1, Square::B5, 4);
        let m4 = Move::new(Square::A7, Square::A6, 0);

        let mut board = Board::default();

        board.make_pl_move::<false>(m1);
        board.make_pl_move::<false>(m2);
        board.make_pl_move::<false>(m3);

        assert_eq!(
            board.count_material(),
            -1,
            "Black lost a pawn and should evaluate to -1.0"
        );

        board.make_pl_move::<false>(m4);
        assert_eq!(
            board.count_material(),
            1,
            "White won a pawn and should evalute to 1.0"
        );
    }

    #[test]
    fn white_loses_queen() {
        let m1 = Move::new(Square::E2, Square::E4, Move::DOUBLE_PAWN);
        let m2 = Move::new(Square::G8, Square::F6, Move::QUIET);
        let m3 = Move::new(Square::D1, Square::G4, Move::QUIET);
        let m4 = Move::new(Square::F6, Square::G4, Move::CAPTURE);
        let m5 = Move::new(Square::A2, Square::A3, Move::QUIET);

        let mut board = Board::default();

        board.make_pl_move::<false>(m1);
        board.make_pl_move::<false>(m2);
        board.make_pl_move::<false>(m3);
        board.make_pl_move::<false>(m4);

        assert_eq!(
            board.count_material(),
            -9,
            "White lost its queen should be -9.0"
        );

        board.make_pl_move::<false>(m5);
        assert_eq!(
            board.count_material(),
            9,
            "Black won whites queen should be -9.0"
        );
    }


  

    fn compare_games(game_1: &Board, game_2: &Board) {
    
        assert_eq!(game_1.fullmove_counter, game_2.fullmove_counter, "Full move counters dont match");
        assert_eq!(game_1.get_pieces(), game_2.get_pieces(), "Piece boards dont match");
        assert_eq!(game_1.get_all_bitboards(), game_2.get_all_bitboards(), "Bitboards dont match");
        assert_eq!(game_1.white_pieces(), game_2.white_pieces(), "White bb does not match");
        assert_eq!(game_1.get_enpassant(), game_2.get_enpassant(), "En passant does not match");
        assert_eq!(game_1.get_halfmoves(), game_2.get_halfmoves(), "Halfmoves dont not match");
        assert_eq!(game_1.black_pieces(), game_2.black_pieces(), "Black bb does not match");
        assert_eq!(game_1.get_occupied(), game_2.get_occupied(), "Occupied does not match");
        assert_eq!(game_1.get_turn(), game_2.get_turn(), "Turn does not match");
        assert_eq!(game_1.get_hash(), game_2.get_hash(), "Hash does not match");
        assert_eq!(game_1.get_castling_rights(), game_2.get_castling_rights(), "Castling rights do not match");
        assert_eq!(game_1.positions.half_move_iter(game_1.get_halfmoves() as u64), game_2.positions.half_move_iter(game_2.get_halfmoves() as u64), "Full move counters dont match");
    }

    #[test]
    fn make_unmake_quiet() {
        let mut board = Board::default();
        let inital_game = board.clone();
        let m = Move::from_string("e2e3", &board).unwrap();
        assert!(board.make_pl_move::<false>(m));
        board.unmake_pl_move(m);
        compare_games(&board, &inital_game);
    }



    #[test]
    fn make_unmake_capture() {
        let mut board = Board::default();
        let m = Move::from_string("e2e3", &board).unwrap();
        assert!(board.make_pl_move::<false>(m));
        let m1 = Move::from_string("b7b5", &board).unwrap();
        assert!(board.make_pl_move::<false>(m1));
        let m2 = Move::from_string("f1b5", &board).unwrap();

        let inital_game = board.clone();
        assert!(board.make_pl_move::<false>(m2));
        board.unmake_pl_move(m2);
      
    
        compare_games(&board, &inital_game);
    }



    #[test]
    fn make_unmake_dpuble_pawn_0() {
        let mut board = Board::default();
        let m = Move::from_string("e2e4", &board).unwrap();
        let inital_game = board.clone();
        assert!(board.make_pl_move::<false>(m));
        board.unmake_pl_move(m);
    
        compare_games(&board, &inital_game);
    }

    #[test]
    fn make_unmake_dpuble_pawn_1() {
        let mut board = Board::default();
        let m = Move::from_string("d2d4", &board).unwrap();
        let inital_game = board.clone();
        assert!(board.make_pl_move::<false>(m));
        board.unmake_pl_move(m);
    
        compare_games(&board, &inital_game);
    }

    #[test]
    fn make_unmake_en_passant() {
        let mut board = Board::from_fen("rnbqkbnr/ppp1pppp/8/8/2PpP3/5P2/PP1P2PP/RNBQKBNR b KQkq c3 0 3").unwrap();
        let mut initial_game = board.clone();
        let en_passant = Move::from_string("d4c3", &board).unwrap();
        assert!(board.make_pl_move::<false>(en_passant));
        board.unmake_pl_move(en_passant);

        compare_games(&board, &initial_game);
    }



    #[test]
    fn unmake_castle() {
        let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
        let mut initial_game = board.clone();
        let castle = Move::from_string("e1g1", &board).unwrap();
        assert!(board.make_pl_move::<false>(castle));
        board.unmake_pl_move(castle);
    
        compare_games(&board, &initial_game);
    }

    

    #[test]
    fn unmake_simple_promo() {
        let mut board = Board::from_fen("5k2/4P3/5K2/8/8/8/8/8 w - - 0 1").unwrap();
        let initial_game = board.clone();
        let promotion = Move::from_string("e7e8q", &board).unwrap();
        assert!(board.make_pl_move::<false>(promotion));
    
        board.unmake_pl_move(promotion);
       
        
        compare_games(&board, &initial_game);
    }
    

    #[test]
    fn unmake_promo_cap() {
        let mut board = Board::from_fen("3n1k2/4P3/5K2/8/8/8/8/8 w - - 0 1").unwrap();
        let initial_game = board.clone();
        let promotion = Move::from_string("e7d8q", &board).unwrap();
        assert!(board.make_pl_move::<false>(promotion));
      
        board.unmake_pl_move(promotion);
       
        
        compare_games(&board, &initial_game);
    }


    #[test]
    fn unmake_multiple_quiets() {
        let mut board = Board::default();
        let mut game_state_1 = board.clone();
        let m1= Move::from_string("e2e3", &board).unwrap();
        assert!(board.make_pl_move::<false>(m1));

        let game_state_2 = board.clone();
        let m2 =  Move::from_string("e7e6", &board).unwrap();
        assert!(board.make_pl_move::<false>(m2));

        let game_state_3 = board.clone();
        let m3 =  Move::from_string("g1f3", &board).unwrap();
        assert!(board.make_pl_move::<false>(m3));

        board.unmake_pl_move(m3);
        compare_games(&board, &game_state_3);
        board.unmake_pl_move(m2);
        compare_games(&board, &game_state_2);
        board.unmake_pl_move(m1);
        compare_games(&board, &game_state_1);
    }

    #[test]
    fn make_draw() {
        let mut board = Board::default();
        board.make_pl_move_from_string::<true>("g1f3");
        board.make_pl_move_from_string::<true>("g8f6");
        board.make_pl_move_from_string::<true>("f3g1");
        board.make_pl_move_from_string::<true>("f6g8");
        assert_eq!(
            board.can_claim_draw(),
            false,
            "Should not be draw"
        );
        board.make_pl_move_from_string::<true>("g1f3");
        board.make_pl_move_from_string::<true>("g8f6");
        board.make_pl_move_from_string::<true>("f3g1");
        board.make_pl_move_from_string::<true>("f6g8");
        assert_eq!(
            board.can_claim_draw(),
            false,
            "Should not be draw"
        );

        board.make_pl_move_from_string::<true>("g1f3");
        board.make_pl_move_from_string::<true>("g8f6");
        board.make_pl_move_from_string::<true>("f3g1");
        board.make_pl_move_from_string::<true>("f6g8");


        assert_eq!(
            board.can_claim_draw(),
            true,
            "Should be draw"
        );
    }

}
