#![feature(rustc_private)]

extern crate rand;
extern crate prusti_contracts;

use rand::{ThreadRng, Rng};

struct RandWrapper {
    rng: ThreadRng
}

impl RandWrapper {
    #[trusted]
    #[ensures="result >= min && result < max"]
    pub fn gen_range(&mut self, min: u32, max: u32) -> u32 {
        self.rng.gen_range(min, max)
    }
}

#[ensures="result >= 0 && result < 10"]
fn func() -> u32{
    let mut rw = RandWrapper { rng: rand::thread_rng() };
    rw.gen_range(0, 10)
}

fn main() {

}
