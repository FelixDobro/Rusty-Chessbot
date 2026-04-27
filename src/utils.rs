

use chess::*;
use chess::{Rank::*, File::*};


pub fn make_move(board: &Board, src_file: File, src_rank: Rank, dest_file: File, dest_rank: Rank) -> Board {
    board.make_move_new(
        ChessMove::new(
            Square::make_square(src_rank, src_file),
            Square::make_square(dest_rank, dest_file),
            None
        ))
}

pub fn display_board(board: &Board) -> () {
    let symbols = [
        ('P', '♙'),
        ('N', '♘'),
        ('B', '♗'),
        ('R', '♖'),
        ('Q', '♕'),
        ('K', '♔'),
        ('p', '♟'),
        ('n', '♞'),
        ('b', '♝'),
        ('r', '♜'),
        ('q', '♛'),
        ('k', '♚'),
    ];
    println!("  A B C D E F G H");
    for rank in (0..8).rev() {
        print!("{} ", rank + 1);
        for file in 0..8 {
            let square = Square::make_square(Rank::from_index(rank), File::from_index(file));

            let mut symbol = ".";

            if let Some(color) = board.color_on(square) {
                if let Some(piece) = board.piece_on(square) {
                    symbol = match (piece, color) {
                        (Piece::Pawn, Color::White) => "♟",
                        (Piece::Pawn, Color::Black) => "♙",
                        (Piece::Bishop, Color::Black) => "♗",
                        (Piece::Bishop, Color::White) => "♝",
                        (Piece::Knight, Color::Black) => "♘",
                        (Piece::Knight, Color::White) => "♞",
                        (Piece::Rook, Color::Black) => "♖",
                        (Piece::Rook, Color::White) => "♜",
                        (Piece::Queen, Color::Black) => "♕",
                        (Piece::Queen, Color::White) => "♛",
                        (Piece::King, Color::Black) => "♔",
                        (Piece::King, Color::White) => "♚",
                    };
                }
            }

            print!("{} ", symbol);
        }
        println!();
    }
    println!("  A B C D E F G H");
}