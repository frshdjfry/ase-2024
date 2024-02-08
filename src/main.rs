mod ring_buffer;

use std::{fs::File, io::Write};
use hound;
fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn convert_sample(sample: i16) -> f32 {
    sample as f32 / i16::MAX as f32
}

fn main() {
   show_info();

    // Parse command line arguments
    // First argument is input .wav file, second argument is output text file.
    let args: Vec<String> = std::env::args().collect();
    let mut reader = match hound::WavReader::open(&args[1]) {
        Ok(reader) => reader,
        Err(error) => {
            eprintln!("Error opening file: {:?}", error);
            return;
        }
    };

    let spec = reader.spec();
    dbg!(spec.bits_per_sample);
    dbg!(spec.channels);
    dbg!(spec.sample_format);
    dbg!(spec.sample_rate);
    let num_channels = spec.channels as usize;

    let mut output_file = match File::create(&args[2]) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Error creating output file: {:?}", error);
            return;
        }
    };
    let mut frame_samples = Vec::with_capacity(num_channels);
    for (i, iterated_sample) in reader.samples::<i16>().enumerate() {
        match iterated_sample {
            Ok(sample) => {
                frame_samples.push(convert_sample(sample));

                if frame_samples.len() == num_channels {
                    if let Err(e) = writeln!(
                        output_file, "{}",
                        frame_samples.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" ")
                    )
                    {
                        eprintln!("Error writing to output file: {:?}", e);
                        break;
                    }
                    frame_samples.clear();
                }
            },
            Err(error) => {
                eprintln!("Error reading sample: {:?}", error);
                break;
            }
        }
    }
}
