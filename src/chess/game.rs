

use crate::chess::chessMove::Move;

use super::chessMove::MOVE_GEN_SIZE;
use super::*;

const GAME_POSITIONS_SIZE: usize = 1024;

#[derive(Debug, Clone, PartialEq)]
#[repr(align(64))]
pub struct Game {
    board: Board,
    fullmove_counter: u16,
    positions: HashList<GAME_POSITIONS_SIZE>,
}

impl Game {
    pub fn default() -> Self {
        Game {
            board: Board::default(),
            fullmove_counter: 1,
            positions: HashList::new(),
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let mut splitted = fen.split(" ").collect::<Vec<&str>>();
        let mut fullmoves_b = 0;

        if let Some(&fullmove) = splitted.get(6) {
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

    fn unmake_quiet<S: Side>(&mut self, m: Move) {}

    pub fn unmake_pl_move<S: Side>(&mut self, m: Move) {
        debug_assert!(self.board.get_halfmoves() != 0);
        match m.flags() {
            Move::QUIET => {
                self.quiet::<S>(m);
            },
            Move::CAPTURE => {},
            _ => {}
        }
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
    use crate::chess::{Board, game::Game};

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
