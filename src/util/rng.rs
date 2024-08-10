use agb::rng::RandomNumberGenerator;

extern crate alloc;
use alloc::vec::Vec;

pub fn fisher_yates_shuffle_vec_inplace<T>(v : &mut Vec<T>, rng : &mut RandomNumberGenerator) {
    let n = v.len();
    for i in (1..n).rev() {
        let j = (rng.gen().abs() as usize)%(i+1);
        v.swap(i, j);
    }
}

pub fn fisher_yates_shuffle_arr_inplace<T, const N : usize>(v : &mut [T; N], rng : &mut RandomNumberGenerator) {
    for i in (1..N).rev() {
        let j = (rng.gen().abs() as usize)%(i+1);
        v.swap(i, j);
    }
}