const WORDLIST: &str = include_str!("../wordlist/english.txt");

pub fn get_wordlist() -> Vec<&'static str> {
    WORDLIST.lines().map(|line| line.trim()).collect()
}
