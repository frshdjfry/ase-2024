use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};

use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

use crate::vibrato::Vibrato;

mod vibrato;
mod comb_filter;
mod ring_buffer;
mod lfo;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 7 {
        eprintln!("Usage: {} <input> <output> <sample_rate> <delay> <depth> <mod_freq>", args[0]);
        std::process::exit(1);
    }
    let input_path = &args[1];
    let output_path = &args[2];
    let sample_rate: f32 = args[3].parse().expect("Invalid sample rate");
    let delay: f32 = args[4].parse().expect("Invalid delay value");
    let depth: f32 = args[5].parse().expect("Invalid depth value");
    let mod_freq: f32 = args[6].parse().expect("Invalid modulation frequency");
    let input_file = File::open(input_path).expect("Failed to open input file");
    let reader = BufReader::new(input_file);
    let mut wav_reader = WavReader::new(reader).expect("Failed to create WAV reader");
    let spec = WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let output_file = File::create(output_path).expect("Failed to create output file");
    let writer = BufWriter::new(output_file);
    let mut wav_writer = WavWriter::new(writer, spec).expect("Failed to create WAV writer");
    let mut vibrato = Vibrato::new(sample_rate, delay, depth, mod_freq, 1.0, 1024).expect("Failed to create Vibrato");
    let buffer_size = 1024;
    let mut samples = wav_reader.samples::<i16>();
    let mut buffer = Vec::with_capacity(buffer_size);
    for sample in samples {
        let sample = sample.expect("Error reading sample");
        buffer.push(sample as f32);
        if buffer.len() == buffer_size {
            let processed_samples = vibrato.process(&buffer);
            for &sample in &processed_samples {
                wav_writer.write_sample(sample as i16).expect("Failed to write sample");
            }
            buffer.clear();
        }
    }
    if !buffer.is_empty() {
        let processed_samples = vibrato.process(&buffer);
        for &sample in &processed_samples {
            wav_writer.write_sample(sample as i16).expect("Failed to write sample");
        }
    }
    wav_writer.finalize().expect("Failed to finalize WAV file");
}
