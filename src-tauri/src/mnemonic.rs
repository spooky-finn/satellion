use rand::Rng;

const WORDLIST: &str = include_str!("../wordlist/english.txt");

fn get_wordlist() -> Vec<&'static str> {
    WORDLIST.lines().map(|line| line.trim()).collect()
}

pub fn generate_random(word_count: usize) -> Vec<&'static str> {
    let words = get_wordlist();
    let mut rng = rand::rng();

    (0..word_count)
        .map(|_| words[rng.random_range(0..words.len())])
        .collect()
}
