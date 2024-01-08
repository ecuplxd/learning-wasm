use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

mod instr;
mod stack;
pub mod types;

pub mod errors;
pub mod importer;
pub mod inst;
pub mod vm;

pub fn random_str(n: usize) -> String {
    thread_rng()
        .sample_iter(Alphanumeric)
        .take(n)
        .map(char::from)
        .collect()
}
