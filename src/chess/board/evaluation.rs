use crate::chess::board::Board;
use crate::chess::board::bitboard::EMPTY as EMPTY_BB;
use crate::chess::constants::Piece::*;
use crate::chess::constants::{BlackSide, Piece, Side, WhiteSide};
use crate::chess::square::Square;

pub const CHECK_MATE: i16 = i16::MIN + 100;
// Values that cant be reached through normal evaluation / a bound for checking
pub const CHECK_MATE_THRESHOLD: i16 = 32_000;
pub const NEG_INFINITY: i16 = i16::MIN + 2;
pub const POSITIVE_INFINITY: i16 = -NEG_INFINITY;

impl Board {
    pub fn calc_phase(&self) -> i16 {
        let mut val = 0;

        val += 1.min(self.piece_bb[Queen.index() + WhiteSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Queen.index()];
        val += 2.min(self.piece_bb[Bishop.index() + WhiteSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Bishop.index()];
        val += 2.min(self.piece_bb[Rook.index() + WhiteSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Rook.index()];
        val += 2.min(self.piece_bb[Knight.index() + WhiteSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Knight.index()];

        val += 1.min(self.piece_bb[Queen.index() + BlackSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Queen.index()];
        val += 2.min(self.piece_bb[Bishop.index() + BlackSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Bishop.index()];
        val += 2.min(self.piece_bb[Rook.index() + BlackSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Rook.index()];
        val += 2.min(self.piece_bb[Knight.index() + BlackSide::OFFSET].count_ones()) as i16
            * PHASE_VALUES[Knight.index()];

        val
    }

    #[inline(always)]
    pub fn calculate_mg(&self) -> i16 {
        let mut val = 0;

        for i in 0..6 {
            let mut piece_bb = self.piece_bb[WhiteSide::OFFSET + i];
            while piece_bb != EMPTY_BB {
                val += MG[WhiteSide::OFFSET + i][piece_bb.lsb().index()];
                val += MG_VALUE[i];
                piece_bb.pop_lsb();
            }
        }
        for i in 0..6 {
            let mut piece_bb = self.piece_bb[BlackSide::OFFSET + i];
            while piece_bb != EMPTY_BB {
                val -= MG[BlackSide::OFFSET + i][piece_bb.lsb().index()];
                val -= MG_VALUE[i];
                piece_bb.pop_lsb();
            }
        }

        val
    }

    #[inline(always)]
    pub fn calculate_eg(&self) -> i16 {
        let mut val = 0;

        for i in 0..6 {
            let mut piece_bb = self.piece_bb[WhiteSide::OFFSET + i];
            while piece_bb != EMPTY_BB {
                val += EG[WhiteSide::OFFSET + i][piece_bb.lsb().index()];
                val += EG_VALUE[i];
                piece_bb.pop_lsb();
            }
        }
        for i in 0..6 {
            let mut piece_bb = self.piece_bb[BlackSide::OFFSET + i];
            while piece_bb != EMPTY_BB {
                val -= EG[BlackSide::OFFSET + i][piece_bb.lsb().index()];
                val -= EG_VALUE[i];
                piece_bb.pop_lsb();
            }
        }

        val
    }

    pub fn eval(&self) -> i16 {
        let mask = -(self.turn.index() as i32);
        let game_phase_i32 = self.game_phase as i32;
        let mg_i32 = self.eval_mg as i32;
        let eg_i32 = self.eval_eg as i32;
        let eval = (game_phase_i32 * mg_i32 + (24 - game_phase_i32) * eg_i32) / 24;
        ((eval ^ mask) - mask) as i16
    }

    #[inline(always)]
    pub fn add_eval<S: Side>(&mut self, p: Piece, s: Square) {
        self.eval_mg += MG[p.index() + S::OFFSET][s.index()] * S::MULTIPLIER;
        self.eval_eg += EG[p.index() + S::OFFSET][s.index()] * S::MULTIPLIER;
    }

    #[inline(always)]
    pub fn rm_eval<S: Side>(&mut self, p: Piece, s: Square) {
        self.eval_mg -= MG[p.index() + S::OFFSET][s.index()] * S::MULTIPLIER;
        self.eval_eg -= EG[p.index() + S::OFFSET][s.index()] * S::MULTIPLIER;
    }

    #[inline(always)]
    pub fn rm_p_eval<S: Side>(&mut self, p: Piece) {
        self.eval_mg -= MG_VALUE[p.index() + S::OFFSET];
        self.eval_eg -= EG_VALUE[p.index() + S::OFFSET];
    }

    #[inline(always)]
    pub fn add_p_eval<S: Side>(&mut self, p: Piece) {
        self.eval_mg += MG_VALUE[p.index() + S::OFFSET];
        self.eval_eg += EG_VALUE[p.index() + S::OFFSET];
    }

    #[inline(always)]
    pub fn result(&self, in_check: bool) -> i16 {
        return if in_check { CHECK_MATE } else { 0 };
    }
}

pub const PHASE_VALUES: [i16; 7] = [0, 1, 1, 2, 4, 0, 0];

#[inline(always)]
const fn flip(i: usize) -> usize {
    i ^ 56
}

pub static MG: [[i16; 64]; 12] = {
    let mut table = [[0i16; 64]; 12];

    table[Pawn.index() + BlackSide::OFFSET] = [
        0, 0, 0, 0, 0, 0, 0, 0, 98, 134, 61, 95, 68, 126, 34, -11, -6, 7, 26, 31, 65, 56, 25, -20,
        -14, 13, 6, 21, 23, 12, 17, -23, -27, -2, -5, 12, 17, 6, 10, -25, -26, -4, -4, -10, 3, 3,
        33, -12, -35, -1, -20, -23, -15, 24, 38, -22, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    table[Knight.index() + BlackSide::OFFSET] = [
        -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37, 65,
        84, 129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8, -23, -9,
        12, 10, 19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58, -33, -17,
        -28, -19, -23,
    ];

    table[Bishop.index() + BlackSide::OFFSET] = [
        -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40, 35,
        50, 37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15, 15, 14,
        27, 18, 10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
    ];

    table[Rook.index() + BlackSide::OFFSET] = [
        32, 42, 32, 51, 63, 9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45, 61,
        16, -24, -11, 7, 26, 24, 35, -8, -20, -36, -26, -12, -1, 9, -7, 6, -23, -45, -25, -16, -17,
        3, 0, -5, -33, -44, -16, -20, -9, -1, 11, -6, -71, -19, -13, 1, 17, 16, 7, -37, -26,
    ];

    table[Queen.index() + BlackSide::OFFSET] = [
        -28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29, 56,
        47, 57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2, -11,
        -2, -5, 2, 14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31, -50,
    ];

    table[King.index() + BlackSide::OFFSET] = [
        -65, 23, 16, -15, -56, -34, 2, 13, 29, -1, -20, -7, -8, -4, -38, -29, -9, 24, 2, -16, -20,
        6, 22, -22, -17, -20, -12, -27, -30, -25, -14, -36, -49, -1, -27, -39, -46, -44, -33, -51,
        -14, -14, -22, -46, -44, -30, -15, -27, 1, 7, -8, -64, -43, -16, 9, 8, -15, 36, 12, -54, 8,
        -28, 24, 14,
    ];
    let mut piece = 0;
    while piece < 6 {
        let mut sq = 0;
        while sq < 64 {
            table[WhiteSide::OFFSET + piece][sq] = table[BlackSide::OFFSET + piece][flip(sq)];
            sq += 1;
        }
        piece += 1
    }
    table
};

pub static EG: [[i16; 64]; 12] = {
    let mut table = [[0i16; 64]; 12];

    table[Pawn.index() + BlackSide::OFFSET] = [
        0, 0, 0, 0, 0, 0, 0, 0, 178, 173, 158, 134, 147, 132, 165, 187, 94, 100, 85, 67, 56, 53,
        82, 84, 32, 24, 13, 5, -2, 4, 17, 17, 13, 9, -3, -7, -7, -8, 3, -1, 4, 7, -6, 1, 0, -5, -1,
        -8, 13, 8, 8, 10, 13, 0, 2, -7, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    table[Knight.index() + BlackSide::OFFSET] = [
        -58, -38, -13, -28, -31, -27, -63, -99, -25, -8, -25, -2, -9, -25, -24, -52, -24, -20, 10,
        9, -1, -9, -19, -41, -17, 3, 22, 22, 22, 11, 8, -18, -18, -6, 16, 25, 16, 17, 4, -18, -23,
        -3, -1, 15, 10, -3, -20, -22, -42, -20, -10, -5, -2, -20, -23, -44, -29, -51, -23, -15,
        -22, -18, -50, -64,
    ];

    table[Bishop.index() + BlackSide::OFFSET] = [
        -14, -21, -11, -8, -7, -9, -17, -24, -8, -4, 7, -12, -3, -13, -4, -14, 2, -8, 0, -1, -2, 6,
        0, 4, -3, 9, 12, 9, 14, 10, 3, 2, -6, 3, 13, 19, 7, 10, -3, -9, -12, -3, 8, 10, 13, 3, -7,
        -15, -14, -18, -7, -1, 4, -9, -15, -27, -23, -9, -23, -5, -9, -16, -5, -17,
    ];

    table[Rook.index() + BlackSide::OFFSET] = [
        13, 10, 18, 15, 12, 12, 8, 5, 11, 13, 13, 11, -3, 3, 8, 3, 7, 7, 7, 5, 4, -3, -5, -3, 4, 3,
        13, 1, 2, 1, -1, 2, 3, 5, 8, 4, -5, -6, -8, -11, -4, 0, -5, -1, -7, -12, -8, -16, -6, -6,
        0, 2, -9, -9, -11, -3, -9, 2, 3, -1, -5, -13, 4, -20,
    ];

    table[Queen.index() + BlackSide::OFFSET] = [
        -9, 22, 22, 27, 27, 19, 10, 20, -17, 20, 32, 41, 58, 25, 30, 0, -20, 6, 9, 49, 47, 35, 19,
        9, 3, 22, 24, 45, 57, 40, 57, 36, -18, 28, 19, 47, 31, 34, 39, 23, -16, -27, 15, 6, 9, 17,
        10, 5, -22, -23, -30, -16, -16, -23, -36, -32, -33, -28, -22, -43, -5, -32, -20, -41,
    ];

    table[King.index() + BlackSide::OFFSET] = [
        -74, -35, -18, -18, -11, 15, 4, -17, -12, 17, 14, 17, 17, 38, 23, 11, 10, 17, 23, 15, 20,
        45, 44, 13, -8, 22, 24, 27, 26, 33, 26, 3, -18, -4, 21, 24, 27, 23, 9, -11, -19, -3, 11,
        21, 23, 16, 7, -9, -27, -11, 4, 13, 14, 4, -5, -17, -53, -34, -21, -11, -28, -14, -24, -43,
    ];

    let mut piece = 0;
    while piece < 6 {
        let mut sq = 0;
        while sq < 64 {
            table[WhiteSide::OFFSET + piece][sq] = table[BlackSide::OFFSET + piece][flip(sq)];
            sq += 1;
        }
        piece += 1
    }

    table
};

pub const MG_VALUE: [i16; 12] = [82, 337, 365, 477, 1025, 0, -82, -337, -365, -477, -1025, -0];
pub const EG_VALUE: [i16; 12] = [94, 281, 297, 512, 936, 0, -94, -281, -297, -512, -936, -0];

#[cfg(test)]
mod test {
    use crate::chess::constants::Piece::*;
    use crate::chess::constants::{BlackSide, WhiteSide};
    use crate::chess::{
        board::{
            Board,
            evaluation::{EG, MG},
        },
        chess_move::Move,
        constants::{Color, Piece, Side},
        square::Square,
    };

    fn compare_evals(initial_board: &Board, new_board: &Board) {
        assert_eq!(
            initial_board.eval_mg, new_board.eval_mg,
            "Mg does not match"
        );
        assert_eq!(
            initial_board.eval_eg, new_board.eval_eg,
            "Eg does not match"
        );
        assert_eq!(
            initial_board.game_phase, new_board.game_phase,
            "Game phase does not match"
        );
    }

    fn compare_table<S: Side>(p: Piece, s: Square, val: i16, mg: bool) {
        if mg {
            assert_eq!(
                MG[p.index() + S::OFFSET][s.index()],
                val,
                "Value at {:?} of {:?} in mg should be {} for {:?}",
                s,
                p,
                val,
                if S::INDEX == 0 {
                    Color::White
                } else {
                    Color::Black
                }
            )
        } else {
            assert_eq!(
                EG[p.index() + S::OFFSET][s.index()],
                val,
                "Value at {:?} of {:?} in eg should be {} for {:?}",
                s,
                p,
                val,
                if S::INDEX == 0 {
                    Color::White
                } else {
                    Color::Black
                }
            )
        }
    }

    #[test]
    fn test_mg_g6_white() {
        compare_table::<WhiteSide>(Pawn, Square::G6, 25, true);
        compare_table::<WhiteSide>(Knight, Square::G6, 73, true);
        compare_table::<WhiteSide>(Bishop, Square::G6, 37, true);
        compare_table::<WhiteSide>(Rook, Square::G6, 61, true);
        compare_table::<WhiteSide>(Queen, Square::G6, 47, true);
        compare_table::<WhiteSide>(King, Square::G6, 22, true);
    }

    #[test]
    fn test_mg_g6_black() {
        compare_table::<BlackSide>(Pawn, Square::G6.flip(), 25, true);
        compare_table::<BlackSide>(Knight, Square::G6.flip(), 73, true);
        compare_table::<BlackSide>(Bishop, Square::G6.flip(), 37, true);
        compare_table::<BlackSide>(Rook, Square::G6.flip(), 61, true);
        compare_table::<BlackSide>(Queen, Square::G6.flip(), 47, true);
        compare_table::<BlackSide>(King, Square::G6.flip(), 22, true);
    }

    #[test]
    fn test_eg_g6_white() {
        compare_table::<WhiteSide>(Pawn, Square::G6, 82, false);
        compare_table::<WhiteSide>(Knight, Square::G6, -19, false);
        compare_table::<WhiteSide>(Bishop, Square::G6, 0, false);
        compare_table::<WhiteSide>(Rook, Square::G6, -5, false);
        compare_table::<WhiteSide>(Queen, Square::G6, 19, false);
        compare_table::<WhiteSide>(King, Square::G6, 44, false);
    }

    #[test]
    fn test_eg_g6_black() {
        compare_table::<BlackSide>(Pawn, Square::G6.flip(), 82, false);
        compare_table::<BlackSide>(Knight, Square::G6.flip(), -19, false);
        compare_table::<BlackSide>(Bishop, Square::G6.flip(), 0, false);
        compare_table::<BlackSide>(Rook, Square::G6.flip(), -5, false);
        compare_table::<BlackSide>(Queen, Square::G6.flip(), 19, false);
        compare_table::<BlackSide>(King, Square::G6.flip(), 44, false);
    }

    #[test]
    fn make_unmake_quiet() {
        let mut board = Board::default();
        let initial_board = board.clone();
        let m = Move::from_string("e2e3", &board).unwrap();
        board.make_pl_move::<true>(m);
        board.unmake_pl_move(m);
        compare_evals(&initial_board, &board);
    }

    #[test]
    fn make_unmake_double() {
        let mut board = Board::default();
        let initial_board = board.clone();
        let m = Move::from_string("e2e4", &board).unwrap();
        board.make_pl_move::<true>(m);
        board.unmake_pl_move(m);
        compare_evals(&initial_board, &board);
    }

    #[test]
    fn make_unmake_capture() {
        let mut board = Board::default();
        let m = Move::from_string("e2e3", &board).unwrap();
        assert!(board.make_pl_move::<true>(m));
        let m1 = Move::from_string("b7b5", &board).unwrap();
        assert!(board.make_pl_move::<true>(m1));
        let initial_board = board.clone();
        let m2 = Move::from_string("f1b5", &board).unwrap();

        assert!(board.make_pl_move::<true>(m2));
        board.unmake_pl_move(m2);

        compare_evals(&initial_board, &board);
    }

    #[test]
    fn make_unmake_en_passant() {
        let mut board =
            Board::from_fen("rnbqkbnr/ppp1pppp/8/8/2PpP3/5P2/PP1P2PP/RNBQKBNR b KQkq c3 0 3")
                .unwrap();
        let initial_game = board.clone();
        let en_passant = Move::from_string("d4c3", &board).unwrap();
        assert!(board.make_pl_move::<false>(en_passant));
        board.unmake_pl_move(en_passant);

        compare_evals(&board, &initial_game);
    }

    #[test]
    fn unmake_castle() {
        let mut board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap();
        let initial_game = board.clone();
        let castle = Move::from_string("e1g1", &board).unwrap();
        assert!(board.make_pl_move::<false>(castle));
        board.unmake_pl_move(castle);

        compare_evals(&board, &initial_game);
    }

    #[test]
    fn unmake_simple_promo() {
        let mut board = Board::from_fen("5k2/4P3/5K2/8/8/8/8/8 w - - 0 1").unwrap();
        let initial_game = board.clone();
        let promotion = Move::from_string("e7e8q", &board).unwrap();
        assert!(board.make_pl_move::<false>(promotion));
        board.unmake_pl_move(promotion);

        compare_evals(&board, &initial_game);
    }

    #[test]
    fn unmake_promo_cap() {
        let mut board = Board::from_fen("3n1k2/4P3/5K2/8/8/8/8/8 w - - 0 1").unwrap();
        let initial_game = board.clone();
        let promotion = Move::from_string("e7d8q", &board).unwrap();
        assert!(board.make_pl_move::<false>(promotion));

        board.unmake_pl_move(promotion);

        compare_evals(&board, &initial_game);
    }

    #[test]
    fn unmake_multiple_quiets() {
        let mut board = Board::default();
        let game_state_1 = board.clone();
        let m1 = Move::from_string("e2e3", &board).unwrap();
        assert!(board.make_pl_move::<false>(m1));

        let game_state_2 = board.clone();
        let m2 = Move::from_string("e7e6", &board).unwrap();
        assert!(board.make_pl_move::<false>(m2));

        let game_state_3 = board.clone();
        let m3 = Move::from_string("g1f3", &board).unwrap();
        assert!(board.make_pl_move::<false>(m3));

        board.unmake_pl_move(m3);
        compare_evals(&board, &game_state_3);
        board.unmake_pl_move(m2);
        compare_evals(&board, &game_state_2);
        board.unmake_pl_move(m1);
        compare_evals(&board, &game_state_1);
    }

    #[test]
    fn test_eval_0() {
        let mut board = Board::default();
        let initial_mg = board.get_mg();
        let initial_eg = board.get_eg();
        board.make_pl_move_from_string::<true>("e2e3");
        assert_eq!(board.get_mg(), initial_mg + (18));
        assert_eq!(board.get_eg(), initial_eg + (-13));
        let mg_last = board.get_mg();
        let eg_last = board.get_eg();
        board.make_pl_move_from_string::<true>("e7e5");
        assert_eq!(board.get_mg(), mg_last - (32));
        assert_eq!(board.get_eg(), eg_last - (-20));
    }

    #[test]
    fn test_eval_1() {
        let mut board =
            Board::from_fen("rnbqkbnr/ppp1pppp/8/8/2PpP3/5P2/PP1P2PP/RNBQKBNR b KQkq c3 0 3")
                .unwrap();
        let initial_mg = board.get_mg();
        let initial_eg = board.get_eg();
        board.make_pl_move_from_string::<true>("d4c3");
        assert_eq!(board.get_mg(), initial_mg - (-6 + 82) - 6);
        assert_eq!(board.get_eg(), initial_eg - (80 + 94) + 3);
    }

    #[test]
    fn test_eval_white_loses_queen() {
        let m1 = Move::new(Square::E2, Square::E4, Move::DOUBLE_PAWN);
        let m2 = Move::new(Square::G8, Square::F6, Move::QUIET);
        let m3 = Move::new(Square::D1, Square::G4, Move::QUIET);
        let m4 = Move::new(Square::F6, Square::G4, Move::CAPTURE);

        let mut board = Board::default();
        let mut initial_mg = board.get_mg();
        let mut initial_eg = board.get_eg();
        board.make_pl_move::<true>(m1);
        assert_eq!(board.get_mg(), initial_mg + 32);
        assert_eq!(board.get_eg(), initial_eg + (-20));
        initial_mg = board.get_mg();
        initial_eg = board.get_eg();
        board.make_pl_move::<true>(m2);
        assert_eq!(board.get_mg(), initial_mg - 36);
        assert_eq!(board.get_eg(), initial_eg - 47);
        initial_mg = board.get_mg();
        initial_eg = board.get_eg();
        board.make_pl_move::<true>(m3);
        assert_eq!(board.get_mg(), initial_mg + (-7));
        assert_eq!(board.get_eg(), initial_eg + (82));
        initial_mg = board.get_mg();
        initial_eg = board.get_eg();
        board.make_pl_move::<true>(m4);
        assert_eq!(board.get_mg(), initial_mg - (1 + 1025 + 3));
        assert_eq!(board.get_eg(), initial_eg - (11 + 936 + 39));
    }
}
