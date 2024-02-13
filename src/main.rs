use std::env;
use std::io::Write;
use std::process::Command;

use hound;

use comb_filter::{CombFilter, FilterType};

mod comb_filter;
mod ring_buffer;

fn show_info() {
    eprintln!("MUSI-6106 Assignment Executable");
    eprintln!("(c) 2024 Stephen Garrett & Ian Clester");
}

fn main() {
    show_info();

    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("No command line arguments provided. Running tests...");

        let output = Command::new("cargo")
            .arg("test")
            .output()
            .expect("Failed to execute command");

        println!("Status: {}", output.status);
        println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(output.status.code().unwrap_or(1));
    }

    if args.len() < 6 {
        eprintln!("Usage: {} <input wave filename> <output wave filename> <filter type> <gain> <delay in seconds>", args[0]);
        return;
    }

    let filter_type = match args[3].as_str() {
        "FIR" => FilterType::FIR,
        "IIR" => FilterType::IIR,
        _ => {
            eprintln!("Invalid filter type. Please specify 'FIR' or 'IIR'.");
            return;
        }
    };

    let gain: f32 = args[4].parse().expect("Gain should be a floating-point number");
    let delay_secs: f32 = args[5].parse().expect("Delay should be a floating-point number in seconds");

    let mut reader = hound::WavReader::open(&args[1]).expect("Failed to open WAV file");
    let spec = reader.spec();
    let sample_rate = spec.sample_rate as f32;
    let num_channels = spec.channels as usize;

    let mut comb_filter = CombFilter::new(filter_type, delay_secs, sample_rate, num_channels, gain);

    let mut writer = hound::WavWriter::create(&args[2], spec).expect("Failed to create WAV writer");

    let block_size = 1024;
    let mut buffer: Vec<f32> = Vec::with_capacity(block_size * num_channels);

    for sample in reader.samples::<i16>() {
        let sample = sample.unwrap() as f32 / i16::MAX as f32;
        buffer.push(sample);

        if buffer.len() >= block_size * num_channels {
            let mut processed_buffer = vec![0.0; buffer.len()];
            for channel in 0..num_channels {
                let mut input_channel_samples = vec![0.0; block_size];
                let mut output_channel_samples = vec![0.0; block_size];
                for (i, sample) in buffer.iter().enumerate().skip(channel).step_by(num_channels) {
                    input_channel_samples[i / num_channels] = *sample;
                }
                comb_filter.process(&[&input_channel_samples], &mut [&mut output_channel_samples]);
                for (i, &sample) in output_channel_samples.iter().enumerate() {
                    processed_buffer[i * num_channels + channel] = sample;
                }
            }
            for &sample in &processed_buffer {
                writer.write_sample((sample * i16::MAX as f32) as i16).unwrap();
            }

            buffer.clear();
        }
    }
    if !buffer.is_empty() {
        let mut processed_buffer = vec![0.0; buffer.len()];
        for channel in 0..num_channels {
            let remaining_samples = buffer.len() / num_channels;
            let mut input_channel_samples = vec![0.0; remaining_samples];
            let mut output_channel_samples = vec![0.0; remaining_samples];
            for (i, sample) in buffer.iter().enumerate().skip(channel).step_by(num_channels) {
                input_channel_samples[i / num_channels] = *sample;
            }
            comb_filter.process(&[&input_channel_samples], &mut [&mut output_channel_samples]);
            for (i, &sample) in output_channel_samples.iter().enumerate() {
                processed_buffer[i * num_channels + channel] = sample;
            }
        }
        for &sample in &processed_buffer {
            writer.write_sample((sample * i16::MAX as f32) as i16).unwrap();
        }
    }

    writer.finalize().unwrap();
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fir_output_zero_at_feedforward_freq() {
        let sample_rate = 44100.0;
        let freq = 1000.0;
        let duration_secs = 1.0;
        let num_samples = (sample_rate * duration_secs) as usize;
        let sine_wave: Vec<f32> = (0..num_samples).map(|i| {
            let t = i as f32 / sample_rate;
            (2.0 * std::f32::consts::PI * freq * t).sin()
        }).collect();
        let mut fir_filter = CombFilter::new(FilterType::FIR, 1.0 / freq, sample_rate, 1, 0.5);
        let mut output_wave = vec![0.0f32; num_samples];
        for (i, &sample) in sine_wave.iter().enumerate() {
            let input_slice: &[&[f32]] = &[&[sample]];
            let output_slice: &mut [&mut [f32]] = &mut [&mut [output_wave[i]]];
            fir_filter.process(input_slice, output_slice);
        }
        for &sample in output_wave.iter() {
            dbg!(sample.abs());
            assert!(sample.abs() < 1e-4, "FIR filter failed at feedforward frequency");
        }
    }

    #[test]
    fn test_iir_feedback_magnitude() {
        let sample_rate = 44100.0;
        let freq = 1000.0;
        let duration_secs = 1.0;
        let num_samples = (sample_rate * duration_secs) as usize;
        let gain = 0.5;
        let delay_secs = 1.0 / freq;
        let sine_wave: Vec<f32> = (0..num_samples).map(|i| {
            let t = i as f32 / sample_rate;
            (2.0 * std::f32::consts::PI * freq * t).sin()
        }).collect();
        let mut iir_filter = CombFilter::new(FilterType::IIR, delay_secs, sample_rate, 1, gain);
        let block_size = 100;
        let mut output_wave = vec![0.0; num_samples];
        for block_start in (0..num_samples).step_by(block_size) {
            let block_end = usize::min(block_start + block_size, num_samples);
            let input_block = &sine_wave[block_start..block_end];
            let output_block = &mut output_wave[block_start..block_end];

            let input_slice: &[&[f32]] = &[input_block];
            let output_slice: &mut [&mut [f32]] = &mut [output_block];

            iir_filter.process(input_slice, output_slice);
        }
        let output_rms = (output_wave.iter().map(|&x| x.powi(2)).sum::<f32>() / output_wave.len() as f32).sqrt();
        let expected_increase_factor = 1.5;
        let input_rms = (sine_wave.iter().map(|&x| x.powi(2)).sum::<f32>() / sine_wave.len() as f32).sqrt();
        assert!(output_rms > input_rms * expected_increase_factor, "IIR filter feedback magnitude test failed");
    }

    #[test]
    fn test_varying_block_size() {
        let sample_rate = 44100.0;
        let duration_secs = 1.0;
        let num_samples = (sample_rate * duration_secs) as usize;
        let freq = 440.0;
        let input_signal: Vec<f32> = (0..num_samples).map(|i| {
            let t = i as f32 / sample_rate;
            (2.0 * std::f32::consts::PI * freq * t).sin()
        }).collect();
        let block_sizes = vec![64, 128, 256, 512, 1024, 2048];

        let gain = 0.5;
        let delay_secs = 0.005;

        let mut previous_output: Option<Vec<f32>> = None;

        for &block_size in block_sizes.iter() {
            let mut comb_filter = CombFilter::new(FilterType::FIR, delay_secs, sample_rate, 1, gain);

            let mut output_signal = Vec::with_capacity(num_samples);
            for block_start in (0..num_samples).step_by(block_size) {
                let block_end = usize::min(block_start + block_size, num_samples);
                let input_block = &input_signal[block_start..block_end];

                let mut output_block = vec![0.0; input_block.len()];
                let input_slice: &[&[f32]] = &[input_block];
                let output_slice: &mut [&mut [f32]] = &mut [&mut output_block[..]];

                comb_filter.process(input_slice, output_slice);
                output_signal.extend_from_slice(&output_block);
            }
            if let Some(ref prev_output) = previous_output {
                let diff: Vec<f32> = output_signal.iter().zip(prev_output.iter()).map(|(a, b)| (a - b).abs()).collect();
                let max_diff = diff.iter().fold(0.0f32, |a, &b| a.max(b));

                println!("Max diff for block size {}: {}", block_size, max_diff);
                assert!(max_diff < 1e-4, "Varying block size test failed at block size {}", block_size);
            }
            previous_output = Some(output_signal.clone());
        }
    }

    #[test]
    fn test_zero_input_signal() {
        let sample_rate = 44100.0;
        let duration_secs = 1.0;
        let num_samples = (sample_rate * duration_secs) as usize;
        let zero_input_signal: Vec<f32> = vec![0.0; num_samples];
        let gain = 0.5;
        let delay_secs = 0.005;
        for &filter_type in &[FilterType::FIR, FilterType::IIR] {
            let mut comb_filter = CombFilter::new(filter_type, delay_secs, sample_rate, 1, gain);
            let mut output_signal = vec![0.0; num_samples];
            for (i, &sample) in zero_input_signal.iter().enumerate() {
                let input_slice: &[&[f32]] = &[&[sample]];
                let output_slice: &mut [&mut [f32]] = &mut [&mut output_signal[i..i + 1]];
                comb_filter.process(input_slice, output_slice);
            }
            let is_output_zero = output_signal.iter().all(|&sample| sample.abs() < 1e-6);

            assert!(is_output_zero, "Zero input signal test failed for {:?} filter", filter_type);
        }

        println!("Zero input signal test passed for both FIR and IIR filters.");
    }

    #[test]
    fn test_filter_response_to_constant_and_impulse_input() {
        let sample_rate = 44100.0;
        let gain = 1.0;
        let delay_secs = 0.0001;
        let input_samples: Vec<f32> = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let expected_output_fir: Vec<f32> = vec![1.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let expected_output_iir: Vec<f32> = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let block_size = input_samples.len();
        let mut comb_filter_fir = CombFilter::new(FilterType::FIR, delay_secs, sample_rate, 1, gain);
        let mut output_fir = vec![0.0; input_samples.len()];
        let input_slice_fir: &[&[f32]] = &[&input_samples[..block_size]];
        let output_slice_fir: &mut [&mut [f32]] = &mut [&mut output_fir[..block_size]];
        comb_filter_fir.process(input_slice_fir, output_slice_fir);
        assert_eq!(output_fir, expected_output_fir, "FIR filter output does not match the expected values.");
        let mut comb_filter_iir = CombFilter::new(FilterType::IIR, delay_secs, sample_rate, 1, gain);
        let mut output_iir = vec![0.0; input_samples.len()];
        let input_slice_iir: &[&[f32]] = &[&input_samples[..block_size]];
        let output_slice_iir: &mut [&mut [f32]] = &mut [&mut output_iir[..block_size]];
        comb_filter_iir.process(input_slice_iir, output_slice_iir);
        assert_eq!(output_iir, expected_output_iir, "IIR filter output does not match the expected values.");
    }
}