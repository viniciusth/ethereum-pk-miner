use keccak_asm::{Digest, Keccak256};
use secp256k1::{PublicKey, Secp256k1, SecretKey};

/// Parses an eth address hex encoded (lowercase) string into its byte format.
/// Panics on invalid format.
pub fn parse_eth_hex(s: &str, v: &mut [u8]) {
    assert!(s.len() == 42 && v.len() == 20);
    decode_hex(s, v);
}

pub fn decode_hex(s: &str, v: &mut [u8]) {
    s.as_bytes()[2..].chunks(2).enumerate().for_each(|(i, c)| {
        unsafe {
            v[i] = u8::from_str_radix(str::from_utf8_unchecked(c), 16).unwrap();
        }
    });
}

pub fn encode_hex(v: &[u8]) -> String {
    v.iter().map(|c| {
        format!("{c:02x}")
    }).collect()
}

/// Generates the eth address from a source private key.
pub fn addr_from_pk(pk: &[u8], target: &mut [u8]) {
    assert!(pk.len() == 32 && target.len() == 20);
    let secp = Secp256k1::new();
    let sk = SecretKey::from_byte_array(pk.try_into().unwrap()).unwrap();
    let pubk = PublicKey::from_secret_key(&secp, &sk).serialize_uncompressed();
    let mut keccak = Keccak256::new();
    keccak.update(&pubk[1..]);
    let data = keccak.finalize();
    target.copy_from_slice(&data[12..32]);
}


#[cfg(test)]
mod tests {
    use crate::utils::encode_hex;

    use super::{addr_from_pk, decode_hex, parse_eth_hex};

    #[test]
    fn parse() {
        let mut data = [0u8; 20];
        let expected = [
            90, 203, 145, 89, 80, 182, 11, 78, 238, 221, 122, 117, 123, 76, 46, 82, 55, 74, 143, 85,
        ];
        let addr = "0x5acb915950b60b4eeedd7a757b4c2e52374a8f55";
        parse_eth_hex(addr, &mut data);
        assert!(data == expected, "not good: {data:?}");
    }

    #[test]
    fn addr() {
        let pk = "0xB2958CC80529E004F4845D3230A1F98E5C28E93C23B0681C0ACE2BB529A65B99";
        let mut pk_bytes = [0; 32];
        decode_hex(pk, &mut pk_bytes);

        let expected = "0x016c310e1c04422564615aee33fb16be4a2bf4be";
        let mut expected_bytes = [0; 20];
        decode_hex(expected, &mut expected_bytes);

        let mut target = [0; 20];
        addr_from_pk(&pk_bytes, &mut target);
        assert!(target == expected_bytes, "Mismatched: {} VS {}", encode_hex(&target), encode_hex(&expected_bytes));
    }
}
