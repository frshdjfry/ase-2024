use std::env;
use crate::vibrato::Vibrato;

mod vibrato;
mod comb_filter;
mod ring_buffer;
mod lfo;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 6 {
        eprintln!("Usage: {} <input> <output> <delay> <depth> <mod_freq>", args[0]);
        std::process::exit(1);
    }
    let input_path = &args[1];
    let output_path = &args[2];
    let delay: f32 = args[3].parse().expect("Invalid delay value");
    let depth: f32 = args[4].parse().expect("Invalid depth value");
    let mod_freq: f32 = args[5].parse().expect("Invalid modulation frequency");
    let mut reader = hound::WavReader::open(input_path).expect("Failed to open WAV file");
    let spec = reader.spec();
    let sample_rate = spec.sample_rate as f32;
    let channels = spec.channels as usize;
    let mut vibrato = Vibrato::new(sample_rate, delay, depth, mod_freq, 0.2, channels).expect("Failed to create Vibrato");
    let mut writer = hound::WavWriter::create(output_path, spec).expect("Failed to create WAV writer");
    let block_size = 1024;

    // Read audio data and write it to the output text file (one column per channel)
    let mut block = vec![Vec::<f32>::with_capacity(block_size); channels];
    let num_samples = reader.len() as usize;
    for (i, sample) in reader.samples::<i16>().enumerate() {
        let sample = sample.unwrap() as f32 / (1 << 15) as f32; // Convert sample to f32
        block[i % channels].push(sample); // Fill block with incoming samples
        // Check if the block is full or it's the last sample
        if (i % (channels * 1024) == 0 && i != 0) || (i == num_samples - 2) {
            let ins = block.iter().map(|c| c.clone()).collect::<Vec<Vec<f32>>>();
            let mut outs = vibrato.process(&ins);
            for j in 0..(channels * outs[0].len()) {
                let channel_index = j % channels;
                let sample_index = j / channels;
                if sample_index < outs[channel_index].len() {
                    writer.write_sample((outs[channel_index][sample_index] * (1 << 15) as f32) as i32).unwrap();
                }
            }

            for channel in block.iter_mut() {
                channel.clear();
            }
            for channel in outs.iter_mut() {
                channel.clear();
            }
        }
    }

}
