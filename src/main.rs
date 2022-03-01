use std::io;
use std::io::Read;

extern crate sndfile;

use sndfile::SndFileIO;

fn i32_bit_amplitudes(in_byte: u8) -> [i32; 8] {
    let mut out_buf = [0; 8];

    for i in 0..out_buf.len() {
        out_buf[i] = (((in_byte & (1 << i)) >> i) as i32)*2 - 1;
    }

    return out_buf;
}

fn main() {
    let path = "./test.wav";
    let write_options = sndfile::WriteOptions::new(
        sndfile::MajorFormat::WAV,
        sndfile::SubtypeFormat::PCM_U8,
        sndfile::Endian::CPU,
        8000,
        1,
    );
    let mut sndfile = sndfile::OpenOptions::WriteOnly(write_options).from_path(path).unwrap();

    let mut stdin = io::stdin();
    let mut in_buf = [0; 1];
    let mut out_buf = [0; 1];
    let mut delta = 0;
    loop {
        match stdin.read(&mut in_buf) {
            Ok(1) => {
                stdin.read(&mut in_buf).unwrap();
                for bit_amplitude in i32_bit_amplitudes(in_buf[0]) {
                    // whacky 32bit to 8bit conversion? >:(
                    delta = (delta + bit_amplitude)%255;
                    out_buf[0] = delta*8388608;
                    sndfile.write_from_slice(&mut out_buf).unwrap();
                }
            }
            Ok(_) => {
                println!("Written {}", path);
                break;
            }
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
    }
}