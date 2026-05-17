    use core::num;

    use crate::chess::chessMove::Move;

    use super::{*};
    use super::chessMove::MOVE_GEN_SIZE;


    #[derive(Debug)]
    #[repr(align(64))]
    pub struct Game {

        board: Board,
        fullmove_counter: u16,
        positions: HashList<GAME_MOVES_SIZE>,
        move_list: MoveList<GAME_MOVES_SIZE>
    }


    impl Game {

        pub fn default() -> Self {

            Game { 
                board: Board::default(),
                fullmove_counter: 1,
                positions: HashList::new(),
                move_list: MoveList::new()
            }
        }


        pub fn from_fen(fen: &str) -> Result<Self, FenError> {
            let mut splitted = fen.split(" ").collect::<Vec<&str>>();
            let mut fullmoves_b = 0;

            if let Some(&fullmove) = splitted.get(6){
                if let Ok(num) = fullmove.parse::<u16>() {
                    fullmoves_b = num;
                }
            }
            let board = Board::from_fen(fen)?;
            let mut positions = HashList::new();
            positions.push(board.get_hash());
            Ok(
                Game { board, fullmove_counter: fullmoves_b, positions: positions, move_list: MoveList::new() }
            )
        }


        pub fn make_pl_move(&mut self, m: Move) -> bool {
            if self.board.make_pl_move(m) {
                self.fullmove_counter += self.board.turn.index() as u16;
                self.positions.push(self.board.get_hash());
                self.move_list.push(m);
                return true;
            } 
            false
        }   


        #[inline(always)]
        pub fn get_board(&self) -> &Board{
            &self.board
        }

        #[inline(always)]
        pub fn get_positions(&self) -> &HashList<GAME_MOVES_SIZE>{
            &self.positions
        }
        // checks only if the current position has occured three times or more and fifty moves
        pub fn can_claim_draw(&self) -> bool {
            
            let halfmoves = self.board.get_halfmoves() as u64;

            if halfmoves > 99 {return true;}

            let mut num_occurences = 0;
            let current_hash = self.board.get_hash();
        
            for &hash in self.positions.half_move_iter(halfmoves) {
                if current_hash == hash {num_occurences += 1}
            }
        
            num_occurences > 2
        } 


        #[inline(always)]
        pub fn generate_pseudolegals(&self) -> MoveList<MOVE_GEN_SIZE> {
            self.board.generate_pseudolegals()
        }

        // just for debugging and testing
        pub fn make_any_legal_move(&mut self) -> bool {
            
            for m in self.board.generate_pseudolegals().as_slice() {
                if self.board.make_pl_move(*m) {
                    return true;
                }
            }
            false
        }

        #[inline(always)]
        pub fn push_state(&mut self, m: Move, new_board: &Board) {
            self.positions.push(new_board.get_hash());
            self.move_list.push(m);
            self.fullmove_counter += new_board.get_turn().opposite().index() as u16;
        } 

        #[inline(always)]
        pub fn pop_only_state(&mut self, new_board: &Board) {

            self.move_list.pop();
            self.positions.pop();

            self.fullmove_counter -= new_board.get_turn().opposite().index() as u16;

        }

    }