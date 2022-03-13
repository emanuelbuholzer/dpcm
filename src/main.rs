use std::io;
use std::io::Read;

extern crate sndfile;

use sndfile::SndFileIO;
use clap::{ArgEnum, Parser};
use crate::Endianness::{BigEndian, LittleEndian};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, arg_enum)]
    input_endianness: Option<Endianness>,

    #[clap(short, long, parse(from_os_str))]
    output_file: std::path::PathBuf,

    #[clap(long, arg_enum)]
    output_endianness: Option<Endianness>,
}

#[derive(Copy, Clone, PartialEq, Eq, ArgEnum, Debug)]
enum Endianness {
    LittleEndian,
    BigEndian,
}

fn main() {
    let args = Args::parse();

    let input_endianness: Endianness;
    if let Some(endianness) = args.input_endianness {
        input_endianness = endianness;
    } else {
        if cfg!(target_endian = "big") {
            input_endianness = BigEndian;
        } else {
            input_endianness = LittleEndian;
        }
    }

    let output_endianness: sndfile::Endian;
    if let Some(endianness) = args.output_endianness {
        match endianness {
            LittleEndian => output_endianness = sndfile::Endian::Little,
            BigEndian => output_endianness = sndfile::Endian::Big,
        }
    } else {
        output_endianness = sndfile::Endian::CPU;
    }

    let output_path = args.output_file.as_path();
    let write_options = sndfile::WriteOptions::new(
        sndfile::MajorFormat::WAV,
        sndfile::SubtypeFormat::PCM_U8,
        output_endianness,
        8000,
        1,
    );
    let mut sndfile = sndfile::OpenOptions::WriteOnly(write_options)
        .from_path(output_path).unwrap();

    let mut stdin = io::stdin();
    let mut in_buf = [0; 1];
    let mut out_buf = [0; 1];
    let mut delta = 0;
    loop {
        match stdin.read(&mut in_buf) {
            Ok(1) => {
                stdin.read(&mut in_buf).unwrap();
                for bit_amplitude in i32_bit_amplitudes(in_buf[0], input_endianness) {
                    // whacky 32bit to 8bit conversion? >:(
                    delta = (delta + bit_amplitude)%255;
                    out_buf[0] = delta*8388608;
                    sndfile.write_from_slice(&mut out_buf).unwrap();
                }
            }
            Ok(_) => {
                println!("Written {}", args.output_file.to_str().unwrap());
                break;
            }
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    }
}

fn i32_bit_amplitudes(in_byte: u8, endianness: Endianness) -> [i32; 8] {
    match endianness {
        LittleEndian => i32_bit_amplitudes_le(in_byte),
        BigEndian => i32_bit_amplitudes_be(in_byte),
    }
}

fn i32_bit_amplitudes_le(in_byte: u8) -> [i32; 8] {
    let mut out_buf = [0; 8];

    for i in 0..out_buf.len() {
        out_buf[i] = (((in_byte & (1 << i)) >> i) as i32)*2 - 1;
    }

    return out_buf;
}

fn i32_bit_amplitudes_be(in_byte: u8) -> [i32; 8] {
    let mut out_buf = i32_bit_amplitudes_le(in_byte);
    out_buf.reverse();
    return out_buf;
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    fn u8_from_bits_le(bits: [bool; 8]) -> u8 {
        let mut out_byte = 0;
        for i in 0..bits.len() {
            out_byte |= (bits[i] as u8) << i;
        }
        return out_byte;
    }

    fn u8_from_bits_be(bits: [bool; 8]) -> u8 {
        let mut out_byte = 0;
        for i in 0..bits.len() {
            out_byte |= (bits[i] as u8) << (7-i);
        }
        return out_byte;
    }

    #[test]
    fn test_u8_from_bit_array_le() {
        let bits = [true, false, false, false, false, false, false, false];
        assert_eq!(u8_from_bits_le(bits), 0b0000_0001);
    }

    #[test]
    fn test_u8_from_bit_array_be() {
        let bits = [true, false, false, false, false, false, false, false];
        assert_eq!(u8_from_bits_be(bits), 0b1000_0000);
    }

    proptest! {
        #[test]
        fn test_i32_bit_amplitudes_le_all_minus_one_or_one(bits: [bool; 8]) {
            let byte = u8_from_bits_le(bits);
            let out_buf = i32_bit_amplitudes_le(byte);
            for i in 0..out_buf.len() {
                prop_assert!(out_buf[i] == (bits[i] as i32)*2-1);
            }
        }

        #[test]
        fn test_i32_bit_amplitudes_be_all_minus_one_or_one(bits: [bool; 8]) {
            let byte = u8_from_bits_be(bits);
            let out_buf = i32_bit_amplitudes_be(byte);
            for i in 0..out_buf.len() {
                prop_assert!(out_buf[i] == (bits[i] as i32)*2-1);
            }
        }
    }
}