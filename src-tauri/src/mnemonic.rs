use alloy_signer_local::coins_bip39::{English, Wordlist};
use rand::Rng;

pub fn generate_random(word_count: usize) -> Vec<&'static str> {
    let words = English::get_all();
    let mut rng = rand::rng();
    (0..word_count)
        .map(|_| words[rng.random_range(0..words.len())])
        .collect()
}
