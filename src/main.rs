#[allow(dead_code)]

mod chess;
mod search;
mod move_sorting;
mod uci;


use std::error::Error;
use crate::search::ids::IDSearch;
use crate::uci::UCIManager;
use crate::search::simple_search::{NegamaxTT};






fn main() -> Result<(), Box<dyn Error>> {
    
    let negamax = NegamaxTT::new(2u64.pow(25) as usize);
    let search = IDSearch::new(negamax);
    // let mut search = Negamax::new();
    let mut mangager = UCIManager::new(Box::new(search));
    mangager.start_protocol()?;

    // board.get_board().print();
    // board.make_pl_move::<true>(Move::from_string("h7h6", &board).unwrap());
    
    // board.get_board().print();
    // board.make_pl_move::<true>(Move::from_string("c1h6", &board).unwrap());

    // board.get_board().print();
    // let m_critical = Move::from_string("h8h7", &board).unwrap();
    // board.make_pl_move::<true>(m_critical);

    // println!("{:?}", board.undo_info);
    // board.unmake_pl_move(m_critical);

    // let m1 = Move::new(Square::E2, Square::E4, 1);
    // let m2 = Move::new(Square::B7, Square::B5,1);
    // let m3 = Move::new(Square::F1, Square::B5, 4);
    // let m4 = Move::new(Square::A7, Square::A6, 0);
    // let m5 = Move::new(Square::H2, Square::G1, 15);
    // let m6 = Move::new(Square::D5, Square::C5, 0);
    // let m7 = Move::new(Square::G2, Square::G4, 1);
    // let m8 = Move::new(Square::B4, Square::A3, 4);
    // let m9 = Move::new(Square::A1, Square::A3, 4);

    // let repetitive_w_1 = Move::new(Square::E1, Square::E2, 0);
    // let repetitive_w_2 = Move::new(Square::E2, Square::E1, 0);
    // let repetitive_b_1 = Move::new(Square::E8, Square::E7, 0);
    // let repetitive_b_2 = Move::new(Square::E7, Square::E8, 0);



    Ok(())
}
