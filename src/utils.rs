
/// Parses an eth address hex encoded (lowercase) string into its byte format.
/// Panics on invalid format.
pub fn parse_eth_hex(s: &str, v: &mut [u8]) {
    s.as_bytes()[2..].chunks(2).enumerate().for_each(|(i, c)| {
        let a = if c[0] <= b'9' {
            c[0] - b'0'
        } else {
            c[0] - b'a' + 10
        };
        let b = if c[1] <= b'9' {
            c[1] - b'0'
        } else {
            c[1] - b'a' + 10
        };

        v[i] = (a << 4) | b;
    });
}

#[cfg(test)]
mod tests {
    use super::parse_eth_hex;


    #[test]
    fn parse_hex() {
        let mut data = [0u8; 20];
        let expected = [90, 203, 145, 89, 80, 182, 11, 78, 238, 221, 122, 117, 123, 76, 46, 82, 55, 74, 143, 85];
        let addr = "0x5acb915950b60b4eeedd7a757b4c2e52374a8f55";
        parse_eth_hex(addr, &mut data);
        assert!(data == expected, "not good: {data:?}");
    }
}
