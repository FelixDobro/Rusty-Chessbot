use flate2::read::GzDecoder;
use std::{env, fs::File, io::copy, path::Path};

pub fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let final_path = Path::new(&out_dir).join("nnue_weights.bin");

    let compressed = File::open("nnue_weights.bin").expect("Not Found");
    let mut decoder = GzDecoder::new(compressed);

    let mut uncompressed = File::create(&final_path).unwrap();
    copy(&mut decoder, &mut uncompressed).expect("Error unpacking");
}
