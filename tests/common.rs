use std::sync::LazyLock;

use rstest::rstest;
use rstest_reuse::template;




#[template]
#[rstest]
#[case::start_pos("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 5, 4_865_609)]
#[case::start_pos("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1", 5, 9771632)]
#[case::start_pos("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2", 5, 11719785)]
#[case::start_pos("8/8/8/4p1K1/2k1P3/8/8/8 b - - 0 1", 8, 4729839)]
#[case::start_pos("4k2r/6r1/8/8/8/8/3R4/R3K3 w Qk - 0 1", 5, 10534193)]
#[case::start_pos("rnbqkbnr/ppp1pppp/8/8/2PpP3/5P2/PP1P2PP/RNBQKBNR b KQkq c3 0 3", 5, 16571869)]
#[case::start_pos("5rk1/p4Qpp/1p6/3B4/2Pb4/1P4Nq/P2r1P1P/4R1K1 b - - 0 26", 6, 150469809)]
#[case::start_pos("8/6n1/8/8/5K2/8/8/1k6 w - - 0 70", 8, 20335969)]
#[case::start_pos("r2rq1k1/1pp2pb1/p1n1bnpp/4p3/PP2P3/B1P1NNP1/2Q1BP1P/3RR1K1 b - - 4 18", 4, 3885495)]
#[case::start_pos("2rq1rk1/ppnnbppp/4p3/3pP3/3P4/1P1Q1N2/P4PPP/R1B1RNK1 b - - 4 14", 5, 48089521)]
#[case::start_pos("3k4/1p3KNq/4r3/3p4/3PnPP1/8/8/8 w - - 9 63", 7, 28101752)]
#[case::start_pos("8/8/5P2/p1p4k/8/1P6/8/4K3 w - - 0 42", 8, 25956602)]
#[case::start_pos("8/5K2/8/4kPRP/7r/8/8/8 w - - 1 57", 6, 16201298)]
#[case::start_pos("8/6k1/4K3/8/4P2p/5N2/8/8 b - - 1 62", 8, 44363103)]
#[case::start_pos("8/7p/5p2/4pBp1/1p2P1PP/pP3k2/b1K5/8 w - g6 0 43", 7, 18978960)]
#[case::start_pos("5Q2/7k/1p2p2p/3bq1p1/8/8/P4PPP/5BK1 w - - 7 372", 5, 17806255)]
#[case::start_pos("8/8/5p2/5P2/3KNn1p/5k2/8/8 w - - 2 75", 7, 29531719)]
#[case::start_pos("8/8/5B2/8/7k/3BR3/3P2K1/2q5 b - - 6 61", 6, 18179751)]
#[case::start_pos("7k/8/5NKP/6b1/8/8/8/8 w - - 0 71", 8, 106106433)]
#[case::start_pos("r7/p7/2p1b1pR/4kp2/8/3P2P1/P1K1PPBP/8 w - - 1 30", 5, 9936127)]
#[case::start_pos("8/4k3/R7/3KP3/3P3p/8/7r/8 b - - 0 41", 6, 18494919)]
#[case::start_pos("8/2q3pk/7p/p3Rp2/6P1/P1N2P1b/2r4P/4Q2K w - - 3 56", 5, 53027459)]
#[case::start_pos("8/6pp/p1r1R3/8/P3N2P/3k1P2/6PK/8 b - - 0 39", 6, 98018828)]
#[case::start_pos("3Q1n1k/5p1p/7P/2q1PPp1/6N1/2P3PK/8/8 b - - 2 45", 5, 5205643)]
#[case::start_pos("1r6/4rkp1/2R1p2p/3p1p1P/1P1P1b2/P7/1B2RPKP/8 b - - 0 35", 5, 25091157)]
#[case::start_pos("8/7k/4P2P/4P3/8/2P1K3/p4P2/R7 b - - 0 62", 8, 224335333)]
#[case::start_pos("4k3/4P3/2P5/1p3p1P/1B1p4/2b2K2/7P/8 b - - 0 75", 7, 34860360)]
#[case::start_pos("8/2PK4/5k2/R4P2/8/8/p2r4/8 w - - 7 66", 7, 160229394)]
#[case::start_pos("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 6, 11_030_083)]
#[case::start_pos("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 5, 15_833_292)]
#[case::kiwipete("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 4, 4_085_603)]
#[case::symmetry("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 4, 3894594)]
#[case::many_checks("r3k2r/5N2/5n2/8/8/8/8/R3K2R w KQkq - 0 1", 5, 18557290)]
#[case::many_checks("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1", 5, 15833292)]
#[case::many_checks("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 11, 85822924)]
#[case::many_checks("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2", 5, 9296387)]
#[case::many_checks("r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 2 2", 6, 107844586)]
#[case::many_checks("rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w KQ - 3 9", 5, 102218344)]
#[case::many_checks("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 5, 89941194)]
#[case::many_checks("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 5, 31912360)]
#[case::many_checks("2r1k2r/8/8/8/8/8/8/R3K2R w KQk - 0 0", 6, 185959088)]
#[case::many_checks("r3k1r1/8/8/8/8/8/8/R3K2R w KQq - 0 1", 6, 190755813)]
#[case::many_checks("4k3/8/8/8/8/8/8/4K2R b K - 0 1", 6, 899442)]
#[case::many_checks("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1", 6, 179862938)]
#[case::many_checks("r3k2r/8/8/8/8/8/8/1R2K2R b Kkq - 0 1", 6, 198328929)]


#[case::pos_1("8/1k6/8/5N2/8/4n3/8/2K5 w - - 0 1", 6, 2_594_412)]
#[case::pos_2("8/8/4k3/3Nn3/3nN3/4K3/8/8 w - - 0 1", 6, 19_870_403)]
#[case::pos_3("K7/8/2n5/1n6/8/8/8/k6N w - - 0 1", 6, 588_695)]
#[case::pos_4("k7/8/2N5/1N6/8/8/8/K6n w - - 0 1", 6, 688_780)]
#[case::pos_5("8/1n4N1/2k5/8/8/5K2/1N4n1/8 b - - 0 1", 6, 8_503_277)]
#[case::pos_6("8/1k6/8/5N2/8/4n3/8/2K5 b - - 0 1", 6, 3_147_566)]
#[case::pos_7("8/8/3K4/3Nn3/3nN3/4k3/8/8 b - - 0 1", 6, 4_405_103)]
#[case::pos_8("K7/8/2n5/1n6/8/8/8/k6N b - - 0 1", 6, 688_780)]
#[case::pos_9("k7/8/2N5/1N6/8/8/8/K6n b - - 0 1", 6, 588_695)]
#[case::pos_10("B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1", 6, 22_823_890)]
#[case::pos_11("8/8/1B6/7b/7k/8/2B1b3/7K w - - 0 1", 6, 28_861_171)]
#[case::pos_12("k7/B7/1B6/1B6/8/8/8/K6b w - - 0 1", 6, 7_881_673)]
#[case::pos_13("K7/b7/1b6/1b6/8/8/8/k6B w - - 0 1", 6, 7_382_896)]
#[case::pos_14("B6b/8/8/8/2K5/5k2/8/b6B b - - 0 1", 6, 9_250_746)]
#[case::pos_15("8/8/1B6/7b/7k/8/2B1b3/7K b - - 0 1", 6, 29_027_891)]
#[case::pos_16("k7/B7/1B6/1B6/8/8/8/K6b b - - 0 1", 6, 7_382_896)]
#[case::pos_17("K7/b7/1b6/1b6/8/8/8/k6B b - - 0 1", 6, 7_881_673)]
#[case::pos_18("7k/RR6/8/8/8/8/rr6/7K w - - 0 1", 6, 44_956_585)]
#[case::pos_19("R6r/8/8/2K5/5k2/8/8/r6R w - - 0 1", 6, 525_169_084)]
#[case::pos_20("7k/RR6/8/8/8/8/rr6/7K b - - 0 1", 6, 44_956_585)]
#[case::pos_21("R6r/8/8/2K5/5k2/8/8/r6R b - - 0 1", 6, 524_966_748)]

#[case::pos_1("6kq/8/8/8/8/8/8/7K w - - 0 1", 6, 391_507)]
#[case::pos_2("6KQ/8/8/8/8/8/8/7k b - - 0 1", 6, 391_507)]
#[case::pos_3("K7/8/8/3Q4/4q3/8/8/7k w - - 0 1", 6, 3_370_175)]
#[case::pos_4("6qk/8/8/8/8/8/8/7K b - - 0 1", 6, 419_369)]
#[case::pos_5("6KQ/8/8/8/8/8/8/7k b - - 0 1", 6, 391_507)]
#[case::pos_6("K7/8/8/3Q4/4q3/8/8/7k b - - 0 1", 6, 3_370_175)]
#[case::pos_7("8/8/8/8/8/K7/P7/k7 w - - 0 1", 6, 6_249)]
#[case::pos_8("8/8/8/8/8/7K/7P/7k w - - 0 1", 6, 6_249)]
#[case::pos_9("K7/p7/k7/8/8/8/8/8 w - - 0 1", 6, 2_343)]
#[case::pos_10("7K/7p/7k/8/8/8/8/8 w - - 0 1", 6, 2_343)]
#[case::pos_11("8/2k1p3/3pP3/3P2K1/8/8/8/8 w - - 0 1", 6, 34_834)]
#[case::pos_12("8/8/8/8/8/K7/P7/k7 b - - 0 1", 6, 2_343)]
#[case::pos_13("8/8/8/8/8/7K/7P/7k b - - 0 1", 6, 2_343)]
#[case::pos_14("K7/p7/k7/8/8/8/8/8 b - - 0 1", 6, 6_249)]
#[case::pos_15("7K/7p/7k/8/8/8/8/8 b - - 0 1", 6, 6_249)]
#[case::pos_16("8/2k1p3/3pP3/3P2K1/8/8/8/8 b - - 0 1", 6, 34_822)]
#[case::pos_17("8/8/8/8/8/4k3/4P3/4K3 w - - 0 1", 6, 11_848)]
#[case::pos_18("4k3/4p3/4K3/8/8/8/8/8 b - - 0 1", 6, 11_848)]
#[case::pos_19("8/8/7k/7p/7P/7K/8/8 w - - 0 1", 6, 10_724)]
#[case::pos_20("8/8/k7/p7/P7/K7/8/8 w - - 0 1", 6, 10_724)]
#[case::pos_21("8/8/3k4/3p4/3P4/3K4/8/8 w - - 0 1", 6, 53_138)]
#[case::pos_22("8/3k4/3p4/8/3P4/3K4/8/8 w - - 0 1", 6, 157_093)]
#[case::pos_23("8/8/3k4/3p4/8/3P4/3K4/8 w - - 0 1", 6, 158_065)]
#[case::pos_24("k7/8/3p4/8/3P4/8/8/7K w - - 0 1", 6, 20_960)]
#[case::pos_25("8/8/7k/7p/7P/7K/8/8 b - - 0 1", 6, 10_724)]
#[case::pos_26("8/8/k7/p7/P7/K7/8/8 b - - 0 1", 6, 10_724)]
#[case::pos_27("8/8/3k4/3p4/3P4/3K4/8/8 b - - 0 1", 6, 53_138)]
#[case::pos_28("8/3k4/3p4/8/3P4/3K4/8/8 b - - 0 1", 6, 158_065)]
#[case::pos_29("8/8/3k4/3p4/8/3P4/3K4/8 b - - 0 1", 6, 157_093)]
#[case::pos_30("k7/8/3p4/8/3P4/8/8/7K b - - 0 1", 6, 21_104)]
#[case::pos_31("7k/3p4/8/8/3P4/8/8/K7 w - - 0 1", 6, 32_191)]
#[case::pos_32("7k/8/8/3p4/8/8/3P4/K7 w - - 0 1", 6, 30_980)]
#[case::pos_33("k7/8/8/7p/6P1/8/8/K7 w - - 0 1", 6, 41_874)]
#[case::pos_34("k7/8/7p/8/8/6P1/8/K7 w - - 0 1", 6, 29_679)]
#[case::pos_35("k7/8/8/6p1/7P/8/8/K7 w - - 0 1", 6, 41_874)]
#[case::pos_36("k7/8/6p1/8/8/7P/8/K7 w - - 0 1", 6, 29_679)]
#[case::pos_37("k7/8/8/3p4/4p3/8/8/7K w - - 0 1", 6, 22_886)]
#[case::pos_38("k7/8/3p4/8/8/4P3/8/7K w - - 0 1", 6, 28_662)]
#[case::pos_39("7k/3p4/8/8/3P4/8/8/K7 b - - 0 1", 6, 32_167)]
#[case::pos_40("7k/8/8/3p4/8/8/3P4/K7 b - - 0 1", 6, 30_749)]
#[case::pos_41("k7/8/8/7p/6P1/8/8/K7 b - - 0 1", 6, 41_874)]
#[case::pos_42("k7/8/7p/8/8/6P1/8/K7 b - - 0 1", 6, 29_679)]
#[case::pos_43("k7/8/8/6p1/7P/8/8/K7 b - - 0 1", 6, 41_874)]
#[case::pos_44("k7/8/6p1/8/8/7P/8/K7 b - - 0 1", 6, 29_679)]
#[case::pos_45("k7/8/8/3p4/4p3/8/8/7K b - - 0 1", 6, 22_579)]
#[case::pos_46("k7/8/3p4/8/8/4P3/8/7K b - - 0 1", 6, 28_662)]
#[case::pos_47("7k/8/8/p7/1P6/8/8/7K w - - 0 1", 6, 41_874)]
#[case::pos_48("7k/8/p7/8/8/1P6/8/7K w - - 0 1", 6, 29_679)]
#[case::pos_49("7k/8/8/1p6/P7/8/8/7K w - - 0 1", 6, 41_874)]
#[case::pos_50("7k/8/1p6/8/8/P7/8/7K w - - 0 1", 6, 29_679)]
#[case::pos_51("k7/7p/8/8/8/8/6P1/K7 w - - 0 1", 6, 55_338)]
#[case::pos_52("k7/6p1/8/8/8/8/7P/K7 w - - 0 1", 6, 55_338)]
#[case::pos_53("3k4/3pp3/8/8/8/8/3PP3/3K4 w - - 0 1", 6, 199_002)]
#[case::pos_54("7k/8/8/p7/1P6/8/8/7K b - - 0 1", 6, 41_874)]
#[case::pos_55("7k/8/p7/8/8/1P6/8/7K b - - 0 1", 6, 29_679)]
#[case::pos_56("7k/8/8/1p6/P7/8/8/7K b - - 0 1", 6, 41_874)]
#[case::pos_57("7k/8/1p6/8/8/P7/8/7K b - - 0 1", 6, 29_679)]
#[case::pos_58("k7/7p/8/8/8/8/6P1/K7 b - - 0 1", 6, 55_338)]
#[case::pos_59("k7/6p1/8/8/8/8/7P/K7 b - - 0 1", 6, 55_338)]
#[case::pos_60("3k4/3pp3/8/8/8/8/3PP3/3K4 b - - 0 1", 6, 199_002)]
#[case::pos_61("8/Pk6/8/8/8/8/6Kp/8 w - - 0 1", 6, 1_030_499)]
#[case::pos_62("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1", 6, 37_665_329)]
#[case::pos_63("8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1", 6, 28_859_283)]
#[case::pos_64("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1", 6, 71_179_139)]
#[case::pos_65("8/Pk6/8/8/8/8/6Kp/8 b - - 0 1", 6, 1_030_499)]
#[case::pos_66("n1n5/1Pk5/8/8/8/8/5Kp1/5N1N b - - 0 1", 6, 37_665_329)]
#[case::pos_67("8/PPPk4/8/8/8/8/4Kppp/8 b - - 0 1", 6, 28_859_283)]
#[case::pos_68("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1", 6, 71_179_139)]
#[case::pos_69("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 6, 11_030_083)]
#[case::pos_70("rnbqkb1r/ppppp1pp/7n/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3", 5, 11_139_762)]
pub fn engine_perf_specs(#[case] fen: &str, #[case] depth: u8, #[case] expected_nodes: usize) {}





pub struct PerftTestCase {
    pub name: String,
    pub fen: String,
    pub depth: u8,
    pub expected: usize,
}

pub fn load_perft_cases() -> Vec<PerftTestCase> {
    include_str!("data.txt") 
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let parts: Vec<&str> = line.split(';').map(|s| s.trim()).collect();
            PerftTestCase {
                name: parts[0].to_string(),
                fen: parts[1].to_string(),
                depth: parts[2].parse().expect("Invalid depth"),
                expected: parts[3].replace('_', "").parse().expect("Invalid node count"),
            }
        })
        .collect()
}

pub static TEST_DATA: LazyLock<Vec<PerftTestCase>> = LazyLock::new(|| load_perft_cases());