use std::sync::LazyLock;

#[derive(Debug, Clone)]
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

pub fn print_test(name: &str, success: bool) {
    println!("test {} ... {}", name, if success {"ok"} else {"Failed!"});
}


