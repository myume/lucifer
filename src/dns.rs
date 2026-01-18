pub fn read_domain(buf: &[u8]) -> String {
    let mut i = 0;
    let mut parts = vec![];
    while buf[i] > 0 {
        let length = buf[i] as usize;
        let part = &buf[i + 1..i + length + 1];
        parts.push(str::from_utf8(part).expect("should be valid utf8"));
        i += length + 1;
    }

    // naively just take the last two parts and join them for the domain
    parts[parts.len() - 2..].join(".")
}

// buf is the buf containing the query. we will write to the same buf
pub fn write_sinkhole_response(buf: &mut [u8]) {
    buf[2] = 0x85;
    buf[3] = 0x83;
    buf[4..12].iter_mut().for_each(|byte| *byte = 0);
    buf[5] = 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_dns_encoded_domain() {
        let input = "github.com";
        let mut encoded = vec![];
        for part in input.split(".") {
            encoded.push(part.len() as u8);
            encoded.extend_from_slice(part.as_bytes());
        }
        encoded.push(0);
        let actual = read_domain(&encoded);
        assert_eq!(input, actual);
    }

    #[test]
    fn test_read_prefix_domain() {
        let input = "www.test.github.com";
        let mut encoded = vec![];
        for part in input.split(".") {
            encoded.push(part.len() as u8);
            encoded.extend_from_slice(part.as_bytes());
        }
        encoded.push(0);
        let actual = read_domain(&encoded);
        assert_eq!("github.com", actual);
    }
}
