use core::{hash, num};

use rstest::rstest;

use crate::chess::{bitboard::{self, EMPTY}, chessMove::Move, constants::Side, square::{self, Square}};
use super::Piece;

use super::Board;
struct JenkinsRng {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

impl JenkinsRng {
    pub const fn new(seed: u64) -> Self {
        let mut rng = JenkinsRng {
            a: 0xf1ea_5eed,
            b: seed,
            c: seed,
            d: seed,
        };

        let mut i = 0;
        while i < 20 {
            rng.next();
            i += 1;
        }
        rng
    }

    const fn next(&mut self) -> u64 {
        let e = self.a.wrapping_sub(self.b.rotate_left(7));
        self.a = self.b ^ self.c.rotate_left(13);
        self.b = self.c.wrapping_add(self.d.rotate_left(37));
        self.c = self.d.wrapping_add(e);
        self.d = e.wrapping_add(self.a);
        self.d
    }
}


#[derive(Debug)]
pub struct ZobristTable {
    pub pieces: [[u64; 64]; 12],
    pub castling: [u64; 16],    
    pub en_passant: [u64; 8],    
    pub side_to_move: u64,       
}


impl ZobristTable {

    const fn new() -> Self {
    
    let mut prng = JenkinsRng::new(12);

    let  piece_nums: [[u64; 64]; 12] = {
        let mut table: [[u64; 64]; 12] = [[0u64; 64]; 12];
        let mut piece = 0;
        while piece < 12 {
            let mut square = 0;
            while square < 64 {
                table[piece][square] = prng.next();
                square += 1;
            }
            piece += 1
        }
        table

        };

    let castling_nums = {
        let mut num_castlings = 0;
        let mut table = [0u64; 16];
        while num_castlings < 16 {
            table[num_castlings] = prng.next();
            num_castlings += 1;
        } 
        table
    };

    let en_passant_nums = {
        let mut num_en_passnats = 0;
        let mut table = [0u64; 8];
        while num_en_passnats < 8 {
            table[num_en_passnats] = prng.next();
            num_en_passnats += 1;
        } 
        table
    };

    let side_to_move_num = prng.next();

    ZobristTable { pieces: piece_nums, castling: castling_nums, en_passant: en_passant_nums, side_to_move: side_to_move_num }

    }
}


pub const ZOBRIST_TABLE: ZobristTable = ZobristTable::new();

impl Board {

    pub fn calculate_hash(&self) -> u64 {

        let mut super_hash = 0u64;
        self.piece_bb
        .into_iter()
        .enumerate()
        .for_each(|(piece_index, mut bb)| {
            while bb != EMPTY {
                let square = bb.lsb().usize();
                super_hash ^=  ZOBRIST_TABLE.pieces[piece_index][square];
                bb.pop_lsb();
            }
            
        });
        
        super_hash ^= ZOBRIST_TABLE.castling[self.castling_rights as usize];
        if self.en_passant != EMPTY {
            super_hash ^= ZOBRIST_TABLE.en_passant[self.en_passant.lsb().file() as usize];
        }
        
        super_hash ^= ZOBRIST_TABLE.side_to_move * self.turn.index() as u64;

        super_hash
    }

    #[inline(always)]
    pub fn update_hash_piece<S: Side>(&mut self, p: Piece, square: Square) {
        self.hash ^= ZOBRIST_TABLE.pieces[p.index() + S::OFFSET][square.usize()]
    }

    #[inline(always)]
    pub fn update_hash_caslte(&mut self, castling: u8) {
        self.hash ^=  ZOBRIST_TABLE.castling[castling as usize];
    }

    #[inline(always)]
    pub fn update_move_hash(&mut self) {
        self.hash ^=  ZOBRIST_TABLE.side_to_move;
    }


    #[inline(always)]
    pub fn update_en_passant_hash(&mut self) {
        if self.en_passant != EMPTY{
            self.hash ^=  ZOBRIST_TABLE.en_passant[self.en_passant.lsb().file() as usize];
        }
    }
}


fn test_1_move_deep(board: &Board, fen: &str) {
    board.generate_pseudolegals().as_sclice()
    .iter()
    .for_each(|&m| {
        let hash_before = board.get_hash();
        if let Some(new_board) = board.make_pl_move_copy(m) {
            assert_eq!(
                new_board.get_hash(),
                new_board.calculate_hash(),
                "Hash inconsistency after move {} in FEN {}", m, fen
        )
        }
    });
}


#[rstest]
#[case::start_pos("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")]
#[case::start_pos("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1")]
#[case::start_pos("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2")]
#[case::start_pos("8/8/8/4p1K1/2k1P3/8/8/8 b - - 0 1")]
#[case::start_pos("4k2r/6r1/8/8/8/8/3R4/R3K3 w Qk - 0 1")]
#[case::start_pos("rnbqkbnr/ppp1pppp/8/8/2PpP3/5P2/PP1P2PP/RNBQKBNR b KQkq c3 0 3")]
#[case::start_pos("5rk1/p4Qpp/1p6/3B4/2Pb4/1P4Nq/P2r1P1P/4R1K1 b - - 0 26")]
#[case::start_pos("8/6n1/8/8/5K2/8/8/1k6 w - - 0 70")]
#[case::start_pos("r2rq1k1/1pp2pb1/p1n1bnpp/4p3/PP2P3/B1P1NNP1/2Q1BP1P/3RR1K1 b - - 4 18")]
#[case::start_pos("2rq1rk1/ppnnbppp/4p3/3pP3/3P4/1P1Q1N2/P4PPP/R1B1RNK1 b - - 4 14")]
#[case::start_pos("3k4/1p3KNq/4r3/3p4/3PnPP1/8/8/8 w - - 9 63")]
#[case::start_pos("8/8/5P2/p1p4k/8/1P6/8/4K3 w - - 0 42")]
#[case::start_pos("8/5K2/8/4kPRP/7r/8/8/8 w - - 1 57")]
#[case::start_pos("3k4/1p3KNq/4r3/3p4/3PnPP1/8/8/8 w - - 9 63")]

fn test_hash_vs_calculated(#[case] fen: &str) {
    let board = Board::from_fen(fen).unwrap();
    board.generate_pseudolegals().as_sclice()
    .iter()
    .for_each(|&m| {
        if let Some(new_board) = board.make_pl_move_copy(m) {
            test_1_move_deep(&new_board, fen);
        }
    });
}
