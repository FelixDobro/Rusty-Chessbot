pub mod board;
pub mod chess_move;
pub mod constants;
pub mod square;



use chess_move::Move;
use chess_move::MOVE_GEN_SIZE;
use board::Board;
use board::hash::HashList;
use board::{*};
use board::bitboard::EMPTY as EMPTY_BB;
use constants::{*};
use constants::Piece::{*};
use chess_move::{*};

use crate::chess::board::bitboard::Bitboard;

const GAME_POSITIONS_SIZE: usize = 1024;


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UndoInfo {
    pub castling_rights: u8,
    pub en_passant_square: Bitboard, 
    pub halfmove_clock: u16,
    pub hash: u64,
    pub captured_piece: Piece,
}

impl UndoInfo {
    
    pub fn empty() -> Self {
        Self { castling_rights: 0, en_passant_square: EMPTY_BB, halfmove_clock: 0, captured_piece: Empty, hash: 0}
    }
}

#[derive(Debug, Clone, PartialEq)]
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


#[derive(Debug, Clone, PartialEq)]
#[repr(align(64))]
pub struct Game {
    board: Board,
    fullmove_counter: u16,
    positions: HashList<GAME_POSITIONS_SIZE>,
    undo_stack: UndoStack,
}

impl Game {
    pub fn default() -> Self {
        let board = Board::default();
        let mut positions = HashList::new();
        positions.push(board.get_hash());
        Game {
            board: board,
            fullmove_counter: 1,
            positions: positions,
            undo_stack: UndoStack::new()
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let mut splitted = fen.split(" ").collect::<Vec<&str>>();
        let mut fullmoves_b = 0;

        if let Some(&fullmove) = splitted.get(5) {
            if let Ok(num) = fullmove.parse::<u16>() {
                fullmoves_b = num;
            }
        }
        let board = Board::from_fen(fen)?;
        let mut positions = HashList::new();
        positions.push(board.get_hash());
        Ok(Game {
            board,
            fullmove_counter: fullmoves_b,
            positions: positions,
            undo_stack: UndoStack::new()
        })
    }

    pub fn make_pl_move_copy(&mut self, m: Move) -> Option<Board> {
        if let Some(mut new_board) = self.board.make_pl_move_copy(m) {
            self.board = new_board;
            self.push_state(&mut new_board);
            return Some(new_board);
        }
        None
    }


    pub fn make_pl_move(&mut self, m: Move) -> bool {
        if let Some(undo_info) = self.board.make_pl_move(m) {
            self.undo_stack.push(undo_info);
            self.positions.push(self.board.get_hash());
            self.fullmove_counter += self.board.get_turn().opposite() as u16;
            return true;
        }
        false
    }


    pub fn unmake_pl_move(&mut self, m: Move) {
        let undo_info = self.undo_stack.pop();
        self.board.unmake_pl_move(m, &undo_info);
        self.positions.pop();
        self.fullmove_counter -= self.board.get_turn() as u16;
    }


    #[inline(always)]
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    #[inline(always)]
    pub fn get_positions(&self) -> &HashList<GAME_MOVES_SIZE> {
        &self.positions
    }
    // checks only if the current position has occured three times or more and fifty moves
    pub fn can_claim_draw(&self) -> bool {
        let halfmoves = self.board.get_halfmoves() as u64;

        if halfmoves > 99 {
            return true;
        }

        let mut num_occurences = 0;
        let current_hash = self.board.get_hash();

        for &hash in self.positions.half_move_iter(halfmoves) {
            if current_hash == hash {
                num_occurences += 1
            }
        }

        num_occurences > 2
    }

    #[inline(always)]
    pub fn generate_pseudolegals(&self) -> MoveList<MOVE_GEN_SIZE> {
        self.board.generate_pseudolegals()
    }

    // just for debugging and testing
    pub fn make_any_legal_move(&mut self) -> bool {
        for &m in self.board.generate_pseudolegals().as_slice() {
            if let Some(mut new_board) = self.board.make_pl_move_copy(m) {
                self.push_state(&mut new_board);
                return true;
            }
        }
        false
    }



    #[inline(always)]
    pub fn push_state(&mut self, new_board: &Board) {
        self.positions.push(new_board.get_hash());
        self.fullmove_counter += new_board.get_turn().opposite().index() as u16;
    }

    #[inline(always)]
    pub fn pop_only_state(&mut self, new_board: &Board) {
        self.positions.pop();
        self.fullmove_counter -= new_board.get_turn().opposite().index() as u16;
    }
}

#[cfg(test)]
mod test {
    use crate::chess::Game;
    use crate::chess::constants::Piece;
    use crate::chess::board::Board;
    use crate::chess::chess_move::Move;

    fn compare_games(game_1: &Game, game_2: &Game) {
    
        assert_eq!(game_1.fullmove_counter, game_2.fullmove_counter, "Full move counters dont match");
        assert_eq!(game_1.board.get_pieces(), game_2.board.get_pieces(), "Piece boards dont match");
        assert_eq!(game_1.board.get_all_bitboards(), game_2.board.get_all_bitboards(), "Bitboards dont match");
        assert_eq!(game_1.board.white_pieces(), game_2.board.white_pieces(), "White bb does not match");
        assert_eq!(game_1.board.get_enpassant(), game_2.board.get_enpassant(), "En passant does not match");
        assert_eq!(game_1.board.get_halfmoves(), game_2.board.get_halfmoves(), "Halfmoves dont not match");
        assert_eq!(game_1.board.black_pieces(), game_2.board.black_pieces(), "Black bb does not match");
        assert_eq!(game_1.board.get_occupied(), game_2.board.get_occupied(), "Occupied does not match");
        assert_eq!(game_1.board.get_turn(), game_2.board.get_turn(), "Turn does not match");
        assert_eq!(game_1.board.get_hash(), game_2.board.get_hash(), "Hash does not match");
        assert_eq!(game_1.board.get_castling_rights(), game_2.board.get_castling_rights(), "Castling rights do not match");
        assert_eq!(game_1.board, game_2.board, "Boards dont match");
        assert_eq!(game_1.positions.half_move_iter(game_1.board.get_halfmoves() as u64), game_2.positions.half_move_iter(game_2.board.get_halfmoves() as u64), "Full move counters dont match");
    }

    #[test]
    fn make_unmake_quiet() {
        let mut game = Game::default();
        let inital_game = game.clone();
        let m = Move::from_string("e2e3", &game).unwrap();
        assert!(game.make_pl_move(m));
        game.unmake_pl_move(m);
        compare_games(&game, &inital_game);
    }



    #[test]
    fn make_unmake_capture() {
        let mut game = Game::default();
        let m = Move::from_string("e2e3", &game).unwrap();
        assert!(game.make_pl_move(m));
        let m1 = Move::from_string("b7b5", &game).unwrap();
        assert!(game.make_pl_move(m1));
        let m2 = Move::from_string("f1b5", &game).unwrap();

        let inital_game = game.clone();
        assert!(game.make_pl_move(m2));
        game.unmake_pl_move(m2);
      
    
        compare_games(&game, &inital_game);
    }



    #[test]
    fn make_unmake_dpuble_pawn_0() {
        let mut game = Game::default();
        let m = Move::from_string("e2e4", &game).unwrap();
        let inital_game = game.clone();
        assert!(game.make_pl_move(m));
        game.unmake_pl_move(m);
    
        compare_games(&game, &inital_game);
    }

    #[test]
    fn make_unmake_dpuble_pawn_1() {
        let mut game = Game::default();
        let m = Move::from_string("d2d4", &game).unwrap();
        let inital_game = game.clone();
        assert!(game.make_pl_move(m));
        game.unmake_pl_move(m);
    
        compare_games(&game, &inital_game);
    }

    #[test]
    fn make_unmake_en_passant() {
        let mut game = Game::from_fen("rnbqkbnr/ppp1pppp/8/8/2PpP3/5P2/PP1P2PP/RNBQKBNR b KQkq c3 0 3").unwrap();
        let mut initial_game = game.clone();
        let en_passant = Move::from_string("d4c3", &game).unwrap();
        assert!(game.make_pl_move(en_passant));
        game.unmake_pl_move(en_passant);

        compare_games(&game, &initial_game);
    }



    #[test]
    fn unmake_castle() {
        let mut game = Game::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
        let mut initial_game = game.clone();
        let castle = Move::from_string("e1g1", &game).unwrap();
        assert!(game.make_pl_move(castle));
        game.unmake_pl_move(castle);
    
        compare_games(&game, &initial_game);
    }

    

    #[test]
    fn unmake_simple_promo() {
        let mut game = Game::from_fen("5k2/4P3/5K2/8/8/8/8/8 w - - 0 1").unwrap();
        let initial_game = game.clone();
        let promotion = Move::from_string("e7e8q", &game).unwrap();
        assert!(game.make_pl_move(promotion));
    
        game.unmake_pl_move(promotion);
       
        
        compare_games(&game, &initial_game);
    }
    

    #[test]
    fn unmake_promo_cap() {
        let mut game = Game::from_fen("3n1k2/4P3/5K2/8/8/8/8/8 w - - 0 1").unwrap();
        let initial_game = game.clone();
        let promotion = Move::from_string("e7d8q", &game).unwrap();
        assert!(game.make_pl_move(promotion));
      
        game.unmake_pl_move(promotion);
       
        
        compare_games(&game, &initial_game);
    }


    #[test]
    fn unmake_multiple_quiets() {
        let mut game = Game::default();
        let mut game_state_1 = game.clone();
        let m1= Move::from_string("e2e3", &game).unwrap();
        assert!(game.make_pl_move(m1));

        let game_state_2 = game.clone();
        let m2 =  Move::from_string("e7e6", &game).unwrap();
        assert!(game.make_pl_move(m2));

        let game_state_3 = game.clone();
        let m3 =  Move::from_string("g1f3", &game).unwrap();
        assert!(game.make_pl_move(m3));

        game.unmake_pl_move(m3);
        compare_games(&game, &game_state_3);
        game.unmake_pl_move(m2);
        compare_games(&game, &game_state_2);
        game.unmake_pl_move(m1);
        compare_games(&game, &game_state_1);
    }


    #[test]
    fn make_unmake_state() {
        let mut game = Game::default();
        let initial_game = game.clone();
        let initial_board = game.board.clone();
        let &m = game.generate_pseudolegals().as_slice().first().unwrap();

        let new_board = game.make_pl_move_copy(m).unwrap();
        game.push_state(&new_board);
        game.pop_only_state(&new_board);

        assert_eq!(game.fullmove_counter, initial_game.fullmove_counter);
        assert_eq!(game.positions.half_move_iter(initial_board.get_halfmoves() as u64), initial_game.positions.half_move_iter(initial_board.get_halfmoves() as u64));
    }
    

    #[test]
    fn make_moves_and_pop() {

        let mut game = Game::default();
        let initial_game = game.clone();
        let initial_board = game.board.clone();
        
        
        let mut board_vec: Vec<Board> = vec![];

        for _ in 0..10 {
            let &m = game.generate_pseudolegals().as_slice().first().unwrap();
            let new_board = game.make_pl_move_copy(m).unwrap();
            board_vec.push(new_board);
        }
        board_vec.iter().rev().for_each(|board| {game.pop_only_state(board);});
        game.board = initial_board;
      
        assert_eq!(initial_game.fullmove_counter, game.fullmove_counter);
        
        assert_eq!(game.positions.half_move_iter(initial_board.get_halfmoves() as u64), initial_game.positions.half_move_iter(initial_board.get_halfmoves() as u64));
    }
}
