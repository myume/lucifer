pub const DNS_HEADER_SIZE: usize = 12;

pub fn read_domain(buf: &[u8]) -> String {
    let mut i = 0;
    let mut parts = vec![];
    while buf[i] > 0 {
        let length = buf[i] as usize;
        let part = &buf[i + 1..i + length + 1];
        parts.push(String::from_utf8_lossy(part));
        i += length + 1;
    }

    // naively just take the last two parts and join them for the domain
    parts[parts.len() - 2..].join(".")
}

// buf is the buf containing the query. we will write to the same buf
pub fn write_sinkhole_response(buf: &mut [u8], query_len: usize) -> usize {
    // Header
    buf[2] = 0x81; // QR=1, RD=1
    buf[3] = 0x80; // RA=1, RCODE=0 (NOERROR)
    buf[4..6].copy_from_slice(&[0x00, 0x01]); // QDCOUNT=1
    buf[6..8].copy_from_slice(&[0x00, 0x01]); // ANCOUNT=1
    buf[8..12].fill(0); // NSCOUNT, ARCOUNT = 0

    // Echo question section back (bytes 12..query_len already in buf)
    let ans = query_len;

    // Answer section
    buf[ans..ans + 2].copy_from_slice(&[0xc0, 0x0c]); // name pointer to offset 12
    buf[ans + 2..ans + 4].copy_from_slice(&[0x00, 0x01]); // TYPE A
    buf[ans + 4..ans + 6].copy_from_slice(&[0x00, 0x01]); // CLASS IN
    buf[ans + 6..ans + 10].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]); // TTL 0
    buf[ans + 10..ans + 12].copy_from_slice(&[0x00, 0x04]); // RDLENGTH 4
    buf[ans + 12..ans + 16].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]); // 0.0.0.0

    ans + 16
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
