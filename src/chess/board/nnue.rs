use bytemuck::{Pod, Zeroable};
use core::arch::x86_64::_mm512_add_epi16;
use core::slice;
use std::arch::x86_64::*;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::ptr::{read, write};

use crate::chess::board::{Board, GAME_POSITIONS_SIZE};
use crate::chess::constants::Color::{Black, White};

use crate::chess::constants::{NUM_PIECES, PIECES, Piece, Side};
use crate::chess::square::Square;

const OPP_PIECE_IDX: [usize; 6] = [5, 6, 7, 8, 9, 10];
const ACCUM_SIZE: usize = 1024;
const INPUT_VEC_DIM: usize = 64 * 64 * 11;
const QA: i16 = 256;
const QB: i8 = 64;

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccumActivated([u8; ACCUM_SIZE]);
unsafe impl Zeroable for AccumActivated {}

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccumVector([i16; ACCUM_SIZE]);

unsafe impl Zeroable for AccumVector {}

impl Deref for AccumVector {
    type Target = [i16; ACCUM_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AccumVector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
struct H1_Bias([i32; 8]);

impl Deref for H1_Bias {
    type Target = [i32; 8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct H1Activated([u8; 8]);
unsafe impl Zeroable for H1Activated {}

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct H2Activated([u8; 32]);
unsafe impl Zeroable for H2Activated {}

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
struct Out_Weight([i8; 32]);

#[repr(C, align(64))]
struct AlignedData<T: ?Sized>(T);

static NNUE_DATA: AlignedData<[u8; 92293696]> = AlignedData(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/nnue_weights.bin"
)));

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NNUE_WEIGHTS {
    accum_weights: [AccumVector; INPUT_VEC_DIM],
    accum_bias: AccumVector,
    h1_weight: [[i8; 2048]; 8],
    h1_bias: H1_Bias,
    h2_weight: [[i8; 32]; 8],
    h2_bias: [i32; 32],
    out_weight: Out_Weight,
    out_bias: i32,
}

#[inline(always)]
pub fn nnue() -> &'static NNUE_WEIGHTS {
    bytemuck::from_bytes(&NNUE_DATA.0)
}

unsafe impl Zeroable for NNUE_WEIGHTS {}
unsafe impl Pod for NNUE_WEIGHTS {}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(align(64))]
pub struct Accumulator(AccumVector);

impl Deref for Accumulator {
    type Target = AccumVector;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Accumulator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Accumulator {
    pub fn own_idx<S: Side>(board: &Board, raw_sq: Square, k_offset: usize) -> usize {
        let p = board.get_piece(raw_sq);
        k_offset + p.index() * 64 + S::flip(raw_sq).index()
    }

    pub fn opp_idx<S: Side>(board: &Board, raw_sq: Square, k_offset: usize) -> usize {
        let p = board.get_piece(raw_sq);
        k_offset + OPP_PIECE_IDX[p.index()] * 64 + S::flip(raw_sq).index()
    }

    pub fn indices<S: Side>(board: &Board) -> IndexList {
        let mut res = IndexList::new();
        let ks = board.get_king_square::<S>();
        let k_offset = S::flip(ks).index() * 64 * 11;
        let mut own_pieces = board.get_side_bb::<S>() & !ks.to_bitboard();
        while !own_pieces.is_empty() {
            let raw_sq = own_pieces.lsb();
            let idx = Self::own_idx::<S>(board, raw_sq, k_offset);
            res.push(idx);
            own_pieces.pop_lsb();
        }

        let mut opp_pieces = board.get_side_bb::<S::OPPOSITE>();
        while !opp_pieces.is_empty() {
            let raw_sq = opp_pieces.lsb();

            let idx = Self::opp_idx::<S>(board, raw_sq, k_offset);
            res.push(idx);
            opp_pieces.pop_lsb();
        }

        res
    }

    pub fn own_p_idx<S: Side>(p: Piece, sq: Square, king: Square) -> usize {
        let k_offset = S::flip(king).index() * 64 * 11;
        k_offset + 64 * p.index() + S::flip(sq).index()
    }

    pub fn opp_p_idx<S: Side>(p: Piece, sq: Square, king: Square) -> usize {
        let k_offset = S::flip(king).index() * 64 * 11;
        k_offset + 64 * OPP_PIECE_IDX[p.index()] + S::flip(sq).index()
    }

    pub fn add_own_piece<S: Side>(&mut self, p: Piece, sq: Square, king: Square) {
        let mut list = IndexList::new();
        list.push(Self::own_p_idx::<S>(p, sq, king));
        self.add_indices(list);
    }

    pub fn add_opp_piece<S: Side>(&mut self, p: Piece, sq: Square, king: Square) {
        let mut list = IndexList::new();
        list.push(Self::opp_p_idx::<S>(p, sq, king));
        self.add_indices(list)
    }

    pub fn rm_opp_piece<S: Side>(&mut self, p: Piece, sq: Square, king: Square) {
        let idx = Self::opp_p_idx::<S>(p, sq, king);
        self.rm_index(idx)
    }

    pub fn rm_own_piece<S: Side>(&mut self, p: Piece, sq: Square, king: Square) {
        let idx = Self::own_p_idx::<S>(p, sq, king);
        self.rm_index(idx)
    }

    #[cfg(target_feature = "avx512bw")]
    pub fn add_indices(&mut self, idx_list: IndexList) {
        unsafe {
            self.add_indices_avx(idx_list);
        }
    }

    #[cfg(not(target_feature = "avx512bw"))]
    pub fn rm_index(&mut self, idx: usize) {
        self.rm_index_fallback(idx);
    }

    #[cfg(target_feature = "avx512bw")]
    pub fn rm_index(&mut self, idx: usize) {
        unsafe { self.rm_index_avx(idx) };
    }

    #[cfg(not(target_feature = "avx512bw"))]
    pub fn add_indices(&mut self, idx_list: IndexList) {
        self.add_indices_fallback();
    }

    pub fn rm_index_fallback(&mut self, idx: usize) {
        let w = nnue().accum_weights[idx];
        for i in 0..ACCUM_SIZE {
            self.0[i] -= w[i];
        }
    }

    #[target_feature(enable = "avx512bw")]
    #[cfg(target_feature = "avx512bw")]
    pub fn rm_index_avx(&mut self, idx: usize) {
        let accum_mem = self.0.as_mut_ptr();
        let w_mem = nnue().accum_weights[idx].as_ptr();
        for offset in (0..ACCUM_SIZE).step_by(64) {
            unsafe {
                let accum_ptr = accum_mem.add(offset);
                let mut accum_1 = _mm512_load_si512(accum_ptr as *const _);
                let mut accum_2 = _mm512_load_si512(accum_ptr.add(32) as *const _);

                let to_sub1 = _mm512_load_si512(w_mem.add(offset) as *const _);
                let to_sub2 = _mm512_load_si512(w_mem.add(offset + 32) as *const _);

                accum_1 = _mm512_sub_epi16(accum_1, to_sub1);
                accum_2 = _mm512_sub_epi16(accum_2, to_sub2);

                _mm512_store_si512(accum_ptr as *mut _, accum_1);
                _mm512_store_si512(accum_ptr.add(32) as *mut _, accum_2);
            }
        }
    }

    #[target_feature(enable = "avx512bw")]
    #[cfg(target_feature = "avx512bw")]
    pub unsafe fn add_indices_avx(&mut self, idx_list: IndexList) {
        let accum_mem = self.0.as_mut_ptr();
        for offset in (0..1024).step_by(64) {
            unsafe {
                let accum_ptr = accum_mem.add(offset);
                let mut accum_1 = _mm512_load_si512(accum_ptr as *const _);
                let mut accum_2 = _mm512_load_si512(accum_ptr.add(32) as *const _);

                for &mem in idx_list.as_slice() {
                    let to_add = mem.as_ptr().add(offset);

                    let to_add1 = _mm512_load_si512(to_add as *const _);
                    let to_add2 = _mm512_load_si512(to_add.add(32) as *const _);

                    accum_1 = _mm512_add_epi16(accum_1, to_add1);
                    accum_2 = _mm512_add_epi16(accum_2, to_add2);
                }
                _mm512_store_si512(accum_ptr as *mut _, accum_1);
                _mm512_store_si512(accum_ptr.add(32) as *mut _, accum_2);
            }
        }
    }

    pub fn add_indices_fallback(&mut self, idx_list: IndexList) {
        for &weight in idx_list.as_slice() {
            (0..1024).for_each(|idx| {
                self.0[idx] += weight[idx];
            })
        }
    }

    #[target_feature(enable = "avx512bw")]
    #[cfg(target_feature = "avx512bw")]
    pub fn accum_screlu(accum: &AccumVector) -> AccumActivated {
        unsafe { Self::accum_screlu_avx(accum) }
    }

    #[cfg(not(target_feature = "avx512bw"))]
    pub fn accum_screlu(accum: &AccumVector) -> AccumActivated {
        accum_screlu_fallback()
    }

    pub fn accum_screlu_fallback(accum: &AccumVector) -> AccumActivated {
        let mut res = AccumActivated::zeroed();
        (0..ACCUM_SIZE).for_each(|idx| {
            let clamped = accum[idx].clamp(0, 255) as u16;
            let squared = clamped * clamped;
            let shifted = squared >> 8;
            res.0[idx] = shifted.min(u8::MAX as u16) as u8;
        });
        res
    }

    #[target_feature(enable = "avx512bw")]
    #[cfg(target_feature = "avx512bw")]
    pub fn accum_screlu_avx(accum: &AccumVector) -> AccumActivated {
        let mut activated: MaybeUninit<AccumActivated> = MaybeUninit::uninit();
        let activated_ptr = activated.as_mut_ptr() as *mut u8;
        let accum_ptr = accum.as_ptr();
        let zero = _mm512_setzero_si512();
        let max_val = _mm512_set1_epi16(255);

        for offset in (0..ACCUM_SIZE).step_by(64) {
            unsafe {
                let acc_offset = accum_ptr.add(offset);
                let acc1 = _mm512_load_si512(acc_offset as *const _);
                let acc2 = _mm512_load_si512(acc_offset.add(32) as *const _);

                let clamped1 = _mm512_min_epi16(max_val, _mm512_max_epi16(acc1, zero));
                let clamped2 = _mm512_min_epi16(max_val, _mm512_max_epi16(acc2, zero));

                let squared1 = _mm512_mullo_epi16(clamped1, clamped1);
                let squared2 = _mm512_mullo_epi16(clamped2, clamped2);

                let shifted1 = _mm512_srli_epi16(squared1, 8);
                let shifted2 = _mm512_srli_epi16(squared2, 8);

                let res1 = _mm512_cvtepi16_epi8(shifted1);
                let res2 = _mm512_cvtepi16_epi8(shifted2);
                let res_p = activated_ptr.add(offset);
                _mm256_store_si256(res_p as *mut _, res1);
                _mm256_store_si256(res_p.add(32) as *mut _, res2);
            }
        }
        unsafe { activated.assume_init() }
    }

    #[cfg(not(target_feature = "avx512bw"))]
    pub fn accum_screlu(accum: &AccumVector) -> AccumActivated {
        accum_screlu_fallback()
    }

    pub fn h1_forward_fallback(input: &AccumActivated, other: &AccumActivated) -> H1Activated {
        let mut out = H1Activated::zeroed();
        for r in 0..8 {
            let mut neuron = nnue().h1_bias[r];
            let weight = nnue().h1_weight[r];
            for w_idx in 0..1024 {
                neuron += (weight[w_idx] as i32) * (input.0[w_idx] as i32);
            }
            for w_idx in 0..1024 {
                neuron += (weight[w_idx + 1024] as i32) * (other.0[w_idx] as i32);
            }
            let activated = (neuron.clamp(0, (QA as i32) * (QB as i32) - 1).pow(2) >> 20) as u8;
            out.0[r] = activated;
        }
        out
    }

    #[target_feature(enable = "avx512bw")]
    #[cfg(target_feature = "avx512bw")]
    pub fn h1_forward(input: &AccumActivated, other: &AccumActivated) -> H1Activated {
        Self::h1_forward_avx(input, other)
    }

    #[target_feature(enable = "avx512bw")]
    #[cfg(target_feature = "avx512bw")]
    pub fn h1_forward_avx(input: &AccumActivated, other: &AccumActivated) -> H1Activated {
        let mut res = H1Activated::zeroed();

        let input_p = input.0.as_ptr() as *const u8;
        let other_p = other.0.as_ptr() as *const u8;

        for weight_row in 0..8 {
            unsafe {
                let w_p = nnue().h1_weight[weight_row].as_ptr();
                let mut cur = _mm512_setzero_si512();

                for offset in (0..1024).step_by(128) {
                    let in1 = _mm512_load_si512(input_p.add(offset) as *const _);
                    let in2 = _mm512_load_si512(input_p.add(offset + 64) as *const _);
                    let w1 = _mm512_load_si512(w_p.add(offset) as *const _);
                    let w2 = _mm512_load_si512(w_p.add(offset + 64) as *const _);

                    cur = _mm512_dpbusd_epi32(cur, in1, w1);
                    cur = _mm512_dpbusd_epi32(cur, in2, w2);
                }

                let w_p_other = w_p.add(1024);
                for offset in (0..1024).step_by(128) {
                    let in1 = _mm512_load_si512(other_p.add(offset) as *const _);
                    let in2 = _mm512_load_si512(other_p.add(offset + 64) as *const _);
                    let w1 = _mm512_load_si512(w_p_other.add(offset) as *const _);
                    let w2 = _mm512_load_si512(w_p_other.add(offset + 64) as *const _);

                    cur = _mm512_dpbusd_epi32(cur, in1, w1);
                    cur = _mm512_dpbusd_epi32(cur, in2, w2);
                }

                let neuron = _mm512_reduce_add_epi32(cur) + nnue().h1_bias.deref()[weight_row];
                let activated = (neuron.clamp(0, QA as i32 * QB as i32 - 1).pow(2) >> 20) as u8;
                res.0[weight_row] = activated;
            }
        }

        res
    }

    pub fn h2_forward_fallback(input: &H1Activated) -> H2Activated {
        let mut res = H2Activated::zeroed();
        let mut temp = nnue().h2_bias;
        let w = nnue().h2_weight;
        for i_idx in 0..w.len() {
            for w_idx in 0..w[0].len() {
                temp[w_idx] += w[i_idx][w_idx] as i32 * input.0[i_idx] as i32;
            }
        }
        for idx in 0..temp.len() {
            let clamped = temp[idx].clamp(0, QA as i32 * QB as i32 - 1);
            let squared = clamped * clamped;
            let shifted = squared >> 20;
            res.0[idx] = shifted as u8;
        }
        res
    }

    #[cfg(target_feature = "avx512bw")]
    pub fn h2_forward(input: &H1Activated) -> H2Activated {
        unsafe { Self::h2_forward_avx(input) }
    }

    #[target_feature(enable = "avx512bw")]
    #[cfg(target_feature = "avx512bw")]
    pub unsafe fn h2_forward_avx(input: &H1Activated) -> H2Activated {
        unsafe {
            let res = H2Activated::zeroed();
            let res_p = res.0.as_ptr();
            let bias_p = nnue().h2_bias.as_ptr();
            let mut low_res = _mm512_load_epi32(bias_p as *const _);
            let mut upper_res = _mm512_load_epi32(bias_p.add(16) as *const _);

            for idx in 0..8 {
                let i = input.0[idx] as i32;
                let broad_cast = _mm512_set1_epi32(i);

                let w_ptr = nnue().h2_weight[idx].as_ptr();
                let w1 = _mm_load_si128(w_ptr as *const _);
                let w2 = _mm_load_si128(w_ptr.add(16) as *const _);

                let w1_vec = _mm512_cvtepi8_epi32(w1);
                let w2_vec = _mm512_cvtepi8_epi32(w2);

                let low = _mm512_mullo_epi32(w1_vec, broad_cast);
                low_res = _mm512_add_epi32(low, low_res);

                let upper = _mm512_mullo_epi32(w2_vec, broad_cast);
                upper_res = _mm512_add_epi32(upper, upper_res);
            }
            let zero = _mm512_setzero_si512();
            let max_val = _mm512_set1_epi32((QA as i32) * (QB as i32) - 1);

            let low_clamped = _mm512_min_epi32(max_val, _mm512_max_epi32(zero, low_res));
            let low_squared = _mm512_mullo_epi32(low_clamped, low_clamped);
            let low_shifted = _mm512_srli_epi32::<20>(low_squared);
            let low_u8 = _mm512_cvtepi32_epi8(low_shifted);
            _mm_store_si128(res_p as *mut _, low_u8);

            let up_clamped = _mm512_min_epi32(max_val, _mm512_max_epi32(zero, upper_res));
            let up_squared = _mm512_mullo_epi32(up_clamped, up_clamped);
            let up_shifted = _mm512_srli_epi32::<20>(up_squared);
            let up_u8 = _mm512_cvtepi32_epi8(up_shifted);
            _mm_store_si128(res_p.add(16) as *mut _, up_u8);

            res
        }
    }

    pub fn output_fallback(input: &H2Activated) -> i32 {
        let weight = nnue().out_weight;
        let mut res = nnue().out_bias;
        for idx in 0..input.0.len() {
            res += input.0[idx] as i32 * weight.0[idx] as i32;
        }
        res
    }

    #[target_feature(enable = "avx512bw")]
    #[cfg(target_feature = "avx512bw")]
    pub fn output(input: &H2Activated) -> i32 {
        unsafe { Self::output_avx(input) }
    }

    #[target_feature(enable = "avx512bw")]
    #[cfg(target_feature = "avx512bw")]
    pub unsafe fn output_avx(input: &H2Activated) -> i32 {
        unsafe {
            let i_ptr = input.0.as_ptr();
            let w_ptr = nnue().out_weight.0.as_ptr();

            let in_vec = _mm256_load_si256(i_ptr as *const _);
            let w_vec = _mm256_load_si256(w_ptr as *const _);

            let dot = _mm256_dpbusd_epi32(_mm256_setzero_si256(), in_vec, w_vec);

            let out_raw = _mm512_reduce_add_epi32(_mm512_castsi256_si512(dot)) + nnue().out_bias;
            out_raw
        }
    }

    pub fn forawrd(&self, other: &Accumulator) -> i32 {
        unsafe {
            let own_act = Self::accum_screlu(&self.0);
            let opp_act = Self::accum_screlu(&other.0);
            let hidden1 = Self::h1_forward(&own_act, &opp_act);
            let hidden2 = Self::h2_forward(&hidden1);
            Self::output(&hidden2)
        }
    }

    pub fn get_accum(&self) -> &AccumVector {
        &self.0
    }

    pub fn new<S: Side>(board: &Board) -> Self {
        let mut res = Self(nnue().accum_bias);
        let indices = Self::indices::<S>(board);
        res.add_indices(indices);
        res
    }

    pub fn reset<S: Side>(&mut self, board: &Board) {
        self.0 = nnue().accum_bias;
        self.add_indices(Self::indices::<S>(board));
    }

    pub fn empty() -> Self {
        Self(AccumVector::zeroed())
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct IndexList {
    values: [&'static [i16; 1024]; 32],
    count: usize,
}

impl IndexList {
    pub fn new() -> Self {
        Self {
            values: [&nnue().accum_weights[0]; 32],
            count: 0,
        }
    }

    pub fn push(&mut self, idx: usize) {
        debug_assert!(self.count < 32, "List is overflowing");
        self.values[self.count] = &nnue().accum_weights[idx];
        self.count += 1;
    }
    pub fn as_slice(&self) -> &[&'static [i16; 1024]] {
        &self.values[0..self.count]
    }
}

#[derive(Debug, Clone)]
pub struct AccumulatorStack {
    values: Box<MaybeUninit<[[Accumulator; 2]; GAME_POSITIONS_SIZE]>>,
    count: usize,
}

impl AccumulatorStack {
    pub fn new() -> Self {
        let values = Box::<[[Accumulator; 2]; GAME_POSITIONS_SIZE]>::new_uninit();
        Self {
            values: values,
            count: 0,
        }
    }

    pub fn peek(&self) -> &[Accumulator; 2] {
        debug_assert!(self.count > 0, "Do not peek from empty Stack");
        unsafe {
            let ptr = self.values.as_ptr() as *const [Accumulator; 2];
            ptr.add(self.count - 1).as_ref().unwrap()
        }
    }

    pub fn push(&mut self, white: Accumulator, black: Accumulator) {
        unsafe {
            let ptr = self.values.as_mut_ptr() as *mut [Accumulator; 2];
            let src = [white, black];
            write(ptr.add(self.count), src);
        }
        self.count += 1
    }

    pub fn as_slice(&self) -> &[[Accumulator; 2]] {
        unsafe {
            let ptr = self.values.as_ptr() as *const [Accumulator; 2];
            slice::from_raw_parts(ptr, self.count)
        }
    }

    pub fn pop(&mut self) {
        self.count -= 1
    }
}

impl PartialEq for AccumulatorStack {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}
impl Eq for AccumulatorStack {}

#[cfg(test)]
mod test {

    use crate::chess::{
        board::{
            Board,
            nnue::{Accumulator, IndexList, nnue},
        },
        chess_move::Move,
        constants::{BlackSide, Side, WhiteSide},
    };

    #[test]
    fn accum_values_1() {
        let board =
            Board::from_fen("2kr1b1r/ppqb1pp1/n1pppn2/7p/7P/N1PPPN2/PPQB1PP1/2KR1B1R b KQkq - 0 1")
                .unwrap();
        let accum = Accumulator::new::<WhiteSide>(&board);
        let mut accum_normal = Accumulator::new::<WhiteSide>(&board);
        accum_normal.0 = nnue().accum_bias;
        accum_normal.add_indices_fallback(Accumulator::indices::<WhiteSide>(&board));

        assert_eq!(accum, accum_normal, "SIMD and fallback not the same");
    }

    #[test]
    fn accum_activation() {
        let board =
            Board::from_fen("2kr1b1r/ppqb1pp1/n1pppn2/7p/7P/N1PPPN2/PPQB1PP1/2KR1B1R b KQkq - 0 1")
                .unwrap();
        let accum = Accumulator::new::<WhiteSide>(&board);

        let optimized = unsafe { Accumulator::accum_screlu(accum.get_accum()) };
        let default = Accumulator::accum_screlu_fallback(accum.get_accum());

        assert_eq!(optimized, default, "SIMD and fallback not the same");
    }

    #[test]
    fn accum_forward() {
        let board =
            Board::from_fen("2kr1b1r/ppqb1pp1/n1pppn2/7p/7P/N1PPPN2/PPQB1PP1/2KR1B1R b KQkq - 0 1")
                .unwrap();
        let cur = Accumulator::new::<WhiteSide>(&board);
        let cur_activated = Accumulator::accum_screlu_fallback(cur.get_accum());
        let opp = Accumulator::new::<BlackSide>(&board);
        let opp_activated = Accumulator::accum_screlu_fallback(opp.get_accum());

        let optimized = unsafe { Accumulator::h1_forward(&cur_activated, &opp_activated) };
        let default = Accumulator::h1_forward_fallback(&cur_activated, &opp_activated);

        assert_eq!(optimized, default, "SIMD and fallback not the same");
    }

    #[test]
    fn h2_forward() {
        let board =
            Board::from_fen("2kr1b1r/ppqb1pp1/n1pppn2/7p/7P/N1PPPN2/PPQB1PP1/2KR1B1R b KQkq - 0 1")
                .unwrap();
        let cur = Accumulator::new::<WhiteSide>(&board);
        let cur_activated = Accumulator::accum_screlu_fallback(cur.get_accum());
        let opp = Accumulator::new::<BlackSide>(&board);
        let opp_activated = Accumulator::accum_screlu_fallback(opp.get_accum());

        let h1_activated = Accumulator::h1_forward_fallback(&cur_activated, &opp_activated);
        let optimized = unsafe { Accumulator::h2_forward(&h1_activated) };
        let default = Accumulator::h2_forward_fallback(&h1_activated);
        assert_eq!(optimized, default, "SIMD and fallback not the same");
    }

    #[test]
    fn out() {
        let board = Board::from_fen("6rk/3q4/2R1pp1p/8/1N1P1P2/2Q5/5KP1/8 b - - 18 66").unwrap();
        let white = Accumulator::new::<WhiteSide>(&board);
        let white_activated = Accumulator::accum_screlu_fallback(white.get_accum());
        let black = Accumulator::new::<BlackSide>(&board);
        let black_activated = Accumulator::accum_screlu_fallback(black.get_accum());

        let h1_activated = Accumulator::h1_forward_fallback(&black_activated, &white_activated);
        let h2_activated = Accumulator::h2_forward_fallback(&h1_activated);

        let optimized = unsafe { Accumulator::output(&h2_activated) };
        let default = Accumulator::output_fallback(&h2_activated);
        let forwarded = black.forawrd(&white);

        assert_eq!(optimized, default, "SIMD and fallback not the same");
        assert_eq!(
            optimized, forwarded,
            "Forward and SIMD/fallback not the same"
        );
    }

    #[test]
    fn add_index() {
        (0..100).for_each(|idx| {
            let mut white = Accumulator::empty();
            let mut copy = Accumulator::empty();

            let mut idx_list = IndexList::new();

            idx_list.push(idx);
            white.add_indices(idx_list.clone());
            copy.add_indices_fallback(idx_list);
            assert_eq!(white, copy, "SIMD and fallback not the same");
        });
    }

    #[test]
    fn rm_idx() {
        (0..100).for_each(|idx| {
            let mut white = Accumulator::empty();

            let mut idx_list = IndexList::new();
            idx_list.push(idx);
            white.add_indices(idx_list);
            let mut clone = white.clone();
            white.rm_index_fallback(idx);
            clone.rm_index(idx);
            assert_eq!(white, clone, "SIMD and fallback not the same");
            assert_eq!(white, Accumulator::empty(), "Accum should be empty");
        });
    }

    #[test]
    fn evaluate_1() {
        let mut b = Board::default();

        b.make_pl_move_from_string::<true>("e2e4");

        let changed_accums = b.accum_stack.peek();

        let white_scratch = Accumulator::new::<WhiteSide>(&b);
        let black_scratch = Accumulator::new::<BlackSide>(&b);

        assert_eq!(
            changed_accums[0], white_scratch,
            "Changed white accum is different than new one"
        );

        assert_eq!(
            changed_accums[1], black_scratch,
            "Changed black accum is different than new one"
        );
    }

    #[test]
    fn evaluate_2() {
        let mut b = Board::default();
        let og = b.clone();
        let m = Move::from_string("e2e4", &b).unwrap();
        b.make_pl_move::<true>(m);
        b.unmake_pl_move(m);
        assert_eq!(b.eval(), og.eval(), "Board evaluates same pos differently");
    }

    #[test]
    fn evaluate_3() {
        let mut b = Board::default();
        let first = b.clone();
        let first_m = Move::from_string("e2e4", &b).unwrap();
        b.make_pl_move::<true>(first_m);
        let second = b.clone();
        let second_m = Move::from_string("e7e5", &b).unwrap();
        b.make_pl_move::<true>(second_m);
        b.unmake_pl_move(second_m);

        assert_eq!(
            b.eval(),
            second.eval(),
            "Board evaluates same pos differently"
        );
        b.unmake_pl_move(first_m);

        assert_eq!(
            b.eval(),
            first.eval(),
            "Board evaluates same pos differently"
        );
    }
}
