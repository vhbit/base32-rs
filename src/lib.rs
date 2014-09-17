#![crate_name="base32"]
#![crate_type="rlib"]

#![feature(phase)]
#![feature(macro_rules)]

#[cfg(test)]
extern crate quickcheck;

#[cfg(test)]
#[phase(plugin)]
extern crate quickcheck_macros;

use std::num::Float;

pub static BASE32_ALPHABET: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

pub fn encode(data: &[u8]) -> Vec<u8> {
    if data.len() == 0 { return Vec::new() };

    let alphabet = BASE32_ALPHABET.as_bytes();
    let mut result = Vec::with_capacity(data.len() * 8 / 5 + 8);
    let mut bits_left = 8;
    let mut bits: u32 = (data[0] as u32) & 0xff;
    let mut iter = data.iter().skip(1);
    let mut stop = false;
    let mut cur_idx = 1u;
    loop {
        if bits_left < 5 {
            if cur_idx < data.len() {
                let ch = data[cur_idx];
                cur_idx += 1;
                bits = (bits << 8) | ((ch as u32) & 0xff);
                bits_left += 8;
            } else {
                if bits_left == 0 {
                    break;
                } else {
                    let leftover = 5 - bits_left;
                    bits = bits << leftover;
                    bits_left += leftover;
                    stop = true;
                }
            }
        }

        let idx = ((bits >> (bits_left - 5)) & 0x1f) as uint;
        bits_left -= 5;
        unsafe { result.push(*alphabet.unsafe_get(idx)) };
        if stop { break };
    }

    let pad: u8 = '='.to_ascii().to_byte();
    while (result.len() % 8) != 0 {
        result.push(pad);
    }

    result
}

static inv_alphabet: [i8, ..256] = [-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 26, 27, 28, 29, 30, 31, -1, -1, -1, -1, -1, -2, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1];

pub fn decode(data: &[u8]) -> Option<Vec<u8>> {
    let mut rem = data.len() % 8;
    if (rem != 0) || (data.len() == 0) {
        return None;
    }

    let mut result = Vec::with_capacity(data.len() * 5 / 8 + 8);
    let mut bits: u32 = inv_alphabet[data[0] as uint] as u32;
    let mut bits_left: u8 = 5;
    let mut iter = data.iter().skip(1);
    let mut stop = false;
    let mut cur_idx: uint = 1;
    loop {
        if bits_left < 8 {
            if cur_idx < data.len() {
                let ch = data[cur_idx];
                cur_idx += 1;

                let v = inv_alphabet[ch as uint];
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
    use super::{encode, decode, BASE32_ALPHABET};
    use quickcheck;
    use std;
    use std::rand::distributions::IndependentSample;

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
