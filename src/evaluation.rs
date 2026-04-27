use chess::*;



pub trait Evaluation {
    fn evaluate(board: &Board) -> f32;

}


pub struct MaterialEvaluator;

impl Evaluation for MaterialEvaluator {
    fn evaluate(board: &Board) -> f32 {
        let mut total = 0.0;

    for idx in 0..64 {
        let square = unsafe {Square::new(idx)};

        if let Some(piece) = board.piece_on(square) {
            let score = match piece {
            
                Piece::Pawn => 1.0,
                Piece::Bishop => 3.0,
                Piece::Knight => 3.0,
                Piece::Rook => 5.0,
                Piece::Queen => 9.0,
                Piece::King => 0.0
            };
            if let Some(color) = board.color_on(square) {
                match color {
                    Color::Black => total -= score,
                    Color::White => total += score,
                };
            }
        }
    }

    total
    }
}



