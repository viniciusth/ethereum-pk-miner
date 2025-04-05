use std::{
    fs::File,
    io::{BufRead, BufReader},
    sync::{Arc, LazyLock},
};

pub static WORDS: LazyLock<Vec<Arc<str>>> = LazyLock::new(|| {
    let mut reader =
        BufReader::new(File::open("./data/english.txt").expect("Should have wordlist"));
    let mut data = Vec::with_capacity(2048);
    for _ in 0..2048 {
        let mut s = String::new();
        reader.read_line(&mut s).expect("Should read word");
        data.push(Arc::from(s));
    }
    data
});

#[cfg(test)]
mod tests {
    use super::WORDS;

    #[test]
    fn words() {
        assert!(WORDS.len() == 2048);
    }
}
