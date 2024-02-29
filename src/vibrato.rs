//! The `Vibrato` module provides functionality to apply a vibrato effect to audio signals.
//! It utilizes a low-frequency oscillator (LFO) to modulate the delay time of the audio signal,
//! creating a varying pitch effect.

use crate::lfo::LFO;
use crate::ring_buffer::RingBuffer;

/// Represents the Vibrato effect with configurable parameters.
pub struct Vibrato {
    delay_line: RingBuffer<f32>,
    lfo: LFO,
    sample_rate: f32,
    delay: f32,
    depth: f32,
}

/// Parameters that can be adjusted in the `Vibrato` effect.
enum VibratoParam {
    SampleRate,
    Delay,
    Depth,
    ModulationFrequency,
}

impl Vibrato {
    /// Creates a new `Vibrato` instance with specified parameters.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The sample rate of the audio signal in Hz.
    /// * `delay` - The base delay time for the vibrato effect in seconds.
    /// * `depth` - The depth of the vibrato modulation in seconds.
    /// * `mod_freq` - The frequency of the modulation oscillator in Hz.
    /// * `amplitude` - The amplitude of the modulation oscillator.
    /// * `wavetable_size` - The size of the wavetable for the LFO.
    ///
    /// # Errors
    ///
    /// Returns an error if the `delay` is less than the `depth`, as this would result in invalid modulation.
    pub fn new(
        sample_rate: f32,
        delay: f32,
        depth: f32,
        mod_freq: f32,
        amplitude: f32,
        wavetable_size: usize,
    ) -> Result<Self, String> {
        if delay < depth {
            return Err("Delay must be greater than or equal to depth".to_string());
        }
        let delay_samples = (delay * sample_rate).round() as usize;
        let depth_samples = (depth * sample_rate).round() as usize;
        let total_size = 2 + delay_samples + depth_samples * 2;

        Ok(Vibrato {
            delay_line: RingBuffer::new(total_size),
            lfo: LFO::new(mod_freq, amplitude, sample_rate, wavetable_size),
            sample_rate,
            delay,
            depth,
        })
    }

    /// Processes an input buffer of audio samples and applies the vibrato effect.
    ///
    /// # Arguments
    ///
    /// * `input` - A slice of input samples to process.
    ///
    /// # Returns
    ///
    /// A `Vec<f32>` containing the processed audio samples with the vibrato effect applied.
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        input.iter().map(|&x| self.process_sample(x)).collect()
    }

    /// Processes a single audio sample and applies the vibrato effect.
    ///
    /// # Arguments
    ///
    /// * `input_sample` - The input sample to process.
    ///
    /// # Returns
    ///
    /// The processed sample with the vibrato effect applied.
    fn process_sample(&mut self, input_sample: f32) -> f32 {
        self.delay_line.push(input_sample);

        let modulation = self.lfo.tick();
        let tap_point = 1.0 + self.delay * self.sample_rate + self.depth * self.sample_rate * modulation;
        let output = self.delay_line.get_frac(tap_point);

        output
    }

    /// Sets the specified parameter to a new value.
    ///
    /// # Arguments
    ///
    /// * `param` - The `VibratoParam` to set.
    /// * `value` - The new value for the parameter.
    ///
    /// # Errors
    ///
    /// Returns an error if setting the parameter would result in invalid configuration,
    /// such as setting the `depth` greater than the `delay`.
    pub fn set_param(&mut self, param: VibratoParam, value: f32) -> Result<(), String> {
        match param {
            VibratoParam::SampleRate => {
                self.sample_rate = value;
            }
            VibratoParam::Delay => {
                if value < self.depth {
                    return Err("Delay must be greater than or equal to depth".to_string());
                }
                self.delay = value;
            }
            VibratoParam::Depth => {
                if value > self.delay {
                    return Err("Depth must be less than or equal to delay".to_string());
                }
                self.depth = value;
            }
            VibratoParam::ModulationFrequency => {
                self.lfo.set_frequency(value);
            }
        }
        Ok(())
    }

    /// Retrieves the current value of the specified parameter.
    ///
    /// # Arguments
    ///
    /// * `param` - The `VibratoParam` to retrieve.
    ///
    /// # Returns
    ///
    /// The current value of the specified parameter.
    pub fn get_param(&self, param: VibratoParam) -> f32 {
        match param {
            VibratoParam::SampleRate => self.sample_rate,
            VibratoParam::Delay => self.delay,
            VibratoParam::Depth => self.depth,
            VibratoParam::ModulationFrequency => self.lfo.get_frequency(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    fn create_default_vibrato() -> Vibrato {
        Vibrato::new(44100.0, 0.005, 0.002, 5.0, 1.0, 1024).unwrap()
    }

    #[test]
    fn test_new_vibrato_success() {
        assert!(Vibrato::new(44100.0, 0.005, 0.002, 5.0, 1.0, 1024).is_ok());
    }

    #[test]
    fn test_new_vibrato_failure() {
        assert!(Vibrato::new(44100.0, 0.002, 0.005, 5.0, 1.0, 1024).is_err());
    }

    #[test]
    fn test_set_param_delay_success() {
        let mut vibrato = create_default_vibrato();
        assert!(vibrato.set_param(VibratoParam::Delay, 0.01).is_ok());
    }

    #[test]
    fn test_set_param_depth_failure() {
        let mut vibrato = create_default_vibrato();
        assert!(vibrato.set_param(VibratoParam::Depth, 0.01).is_err());
    }

    #[test]
    fn test_get_param_sample_rate() {
        let vibrato = create_default_vibrato();
        assert_eq!(vibrato.get_param(VibratoParam::SampleRate), 44100.0);
    }

    #[test]
    fn output_equals_delayed_input_with_zero_modulation() {
        let sample_rate = 44100.0;
        let delay = 0.005;
        let depth = 0.0;
        let mod_freq = 5.0;

        let mut vibrato = Vibrato::new(sample_rate, delay, depth, mod_freq, 0.0, 1024).unwrap();
        let input = vec![1.0; 441];
        let expected_delay_samples = (delay * sample_rate) as usize;

        let output = vibrato.process(&input);
        assert_eq!(output.len(), input.len());


        for i in 0..expected_delay_samples - 200 {
            assert_eq!(output[i], 0.0);
        }
    }

    #[test]
    fn dc_input_results_in_dc_output() {
        let sample_rate = 44100.0;
        let delay = 0.005;
        let depth = 0.002;
        let mod_freq = 5.0;

        let mut vibrato = Vibrato::new(sample_rate, delay, depth, mod_freq, 1.0, 1024).unwrap();
        let input = vec![0.5; 441];

        let output = vibrato.process(&input);
        assert_eq!(output.len(), input.len());


        for &sample in &output {
            assert!((sample - 0.5).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn varying_input_block_size() {
        let sample_rate = 44100.0;
        let delay = 0.005;
        let depth = 0.002;
        let mod_freq = 5.0;

        let mut vibrato = Vibrato::new(sample_rate, delay, depth, mod_freq, 1.0, 1024).unwrap();

        for &size in &[128, 256, 512, 1024] {
            let input = vec![1.0; size];
            let output = vibrato.process(&input);
            assert_eq!(output.len(), size);

        }
    }

    #[test]
    fn zero_input_signal() {
        let sample_rate = 44100.0;
        let delay = 0.005;
        let depth = 0.002;
        let mod_freq = 5.0;

        let mut vibrato = Vibrato::new(sample_rate, delay, depth, mod_freq, 1.0, 1024).unwrap();
        let input = vec![0.0; 1024];

        let output = vibrato.process(&input);
        assert_eq!(output, input);
    }

    #[test]
    fn modulation_depth_impact() {
        let sample_rate = 44100.0;
        let delay = 0.005;
        let mod_freq = 5.0;


        let depth1 = 0.001;
        let depth2 = 0.002;

        let mut vibrato1 = Vibrato::new(sample_rate, delay, depth1, mod_freq, 1.0, 1024).unwrap();
        let mut vibrato2 = Vibrato::new(sample_rate, delay, depth2, mod_freq, 1.0, 1024).unwrap();

        let input = vec![1.0; 1024];
        let output1 = vibrato1.process(&input);
        let output2 = vibrato2.process(&input);


        assert_ne!(output1, output2);
    }
}