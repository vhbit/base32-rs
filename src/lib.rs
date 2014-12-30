#![crate_name="base32"]
#![crate_type="rlib"]

#![feature(phase)]
#![feature(macro_rules)]

#[cfg(test)]
extern crate quickcheck;

#[cfg(test)]
#[phase(plugin)]
extern crate quickcheck_macros;


pub static BASE32_ALPHABET: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

pub fn encode(data: &[u8]) -> Vec<u8> {
    if data.len() == 0 { return Vec::new() };

    let alphabet = BASE32_ALPHABET.as_bytes();
    let mut result = Vec::with_capacity(data.len() * 8 / 5 + 8);

    for chunk in data.chunks(5) {
        if chunk.len() == 5 {
            result.push(alphabet[((chunk[0] & 0xF8) >> 3) as uint]);
            result.push(alphabet[(((chunk[0] & 0x07) << 2) | ((chunk[1] & 0xC0) >> 6)) as uint]);
            result.push(alphabet[((chunk[1] & 0x3E) >> 1) as uint]);
            result.push(alphabet[(((chunk[1] & 0x01) << 4) | ((chunk[2] & 0xF0) >> 4)) as uint]);
            result.push(alphabet[(((chunk[2] & 0x0F) << 1) | (chunk[3] >> 7)) as uint]);
            result.push(alphabet[((chunk[3] & 0x7C) >> 2) as uint]);
            result.push(alphabet[(((chunk[3] & 0x03) << 3) | ((chunk[4] & 0xE0) >> 5)) as uint]);
            result.push(alphabet[(chunk[4] & 0x1F) as uint]);
        } else {
            // Handle leftover, max 40 bits, so 64 should be enough
            let mut leftover: u64 = 0;
            for &ch in chunk.iter() {
                leftover = leftover << 8 | ((ch as u64) & 0xff)
            }

            let total_bits = chunk.len() * 8;
            let padding_bits = (chunk.len() * 8 / 5 + 1) * 5 - total_bits;
            leftover = leftover << padding_bits;

            let mut bits_left = (total_bits + padding_bits) as int;
            while bits_left > 0 {
                result.push(alphabet[((leftover >> (bits_left as uint - 5)) & 0x1f) as uint]);
                bits_left -= 5;
            }
        }
    }

    let pad: u8 = '=' as u8;
    while (result.len() % 8) != 0 {
        result.push(pad);
    }

    result
}

static INV_ALPHABET: [i8, ..256] = [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 26, 27, 28, 29, 30, 31, -1, -1, -1, -1, -1, -2, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1];

pub fn decode(data: &[u8]) -> Option<Vec<u8>> {
    let rem = data.len() % 8;
    if (rem != 0) || (data.len() == 0) {
        return None;
    }

    let mut result = Vec::with_capacity(data.len() * 5 / 8 + 8);
    let mut bits: u32 = INV_ALPHABET[data[0] as uint] as u32;
    let mut bits_left: u8 = 5;
    let mut cur_idx: uint = 1;
    loop {
        if bits_left < 8 {
            if cur_idx < data.len() {
                let ch = data[cur_idx];
                cur_idx += 1;

                let v = INV_ALPHABET[ch as uint];
                if v == -2 {
                    break;
                } else if v == -1 {
                    return None;
                }
                else {
                    bits = (bits << 5) | (v as u32);
                    bits_left += 5;
                }
            }
            else {
                if bits_left == 0 {
                    break;
                } else {
                    return None
                }
            }
        }

        if bits_left >= 8 {
            let v = (bits >> (bits_left - 8) as uint) as u8;
            bits_left -= 8;
            result.push(v);
        }
    }

    Some(result)
}

#[cfg(test)]
mod test {
    extern crate test;
    use super::{encode, decode};

    #[quickcheck]
    fn invertible(data: Vec<u8>) -> bool {
        if data.len() == 0 {
            decode(encode(data.as_slice()).as_slice()).is_none()
        } else {
            decode(encode(data.as_slice()).as_slice()).unwrap() == data
        }
    }

    #[test]
    fn invalid_chars() {
        assert_eq!(decode("ABCDEFG,".as_bytes()), None)
    }

    #[test]
    fn invalid_len() {
        assert_eq!(decode("ABCD".as_bytes()), None)
    }

    #[bench]
    fn bench_encode(b: &mut test::Bencher) {
        let data = [0, 0, 0, 0, 0];
        b.iter(|| encode(data.as_slice()));
        b.bytes = data.len() as u64;
    }

    #[bench]
    fn bench_decode(b: &mut test::Bencher) {
        let data = "ABCDEFGH".as_bytes();
        b.iter(|| decode(data));
        b.bytes = data.len() as u64;
    }
}
