use std::sync::Arc;

use rand::RngCore;

use crate::{measure, wordlist::WORDS};

/// Maybe we'll want to try different rng's, so just leave a trait here for now
pub trait CryptoGenerator {
    fn generate_pk(&mut self) -> [u8; 32];

    fn generate_mnemonic(&mut self) -> Vec<Arc<str>>;
}

impl<T: RngCore> CryptoGenerator for T {
    fn generate_pk(&mut self) -> [u8; 32] {
        let mut data = [0; 32];
        measure! {
            "generate_pk"
            {
                self.fill_bytes(&mut data);
            }
        }
        data
    }

    /// Each word is selected by 11 random bits,
    /// lets choose a mnemonic with either [12, 24] words.
    /// min = 11 * 12 = 132 bits, max = 11 * 24 = 244 bits
    /// just generate pk (256 bits), first bit defines size, rest is used to get the words
    fn generate_mnemonic(&mut self) -> Vec<Arc<str>> {
        let data = self.generate_pk();
        let mut num = data[0] as u32;
        let len = if num & 1 == 0 { 12 } else { 24 };
        num >>= 1;
        let mut bits = 7;
        let mut words = Vec::with_capacity(len);
        for &d in data.iter().skip(1) {
            num = (num << 8) | d as u32;
            bits += 8;
            if bits >= 11 {
                words.push(WORDS[num as usize & 2047].clone());
                num >>= 11;
                bits -= 11;
                if words.len() == len {
                    break;
                }
            }
        }

        words
    }
}
