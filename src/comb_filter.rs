use crate::ring_buffer::RingBuffer;

pub struct CombFilter {
    filter_type: FilterType,
    delay_lines: Vec<RingBuffer<f32>>,
    gain: f32,
    sample_rate: f32,
    buffer: Vec<Vec<f32>>,
    buffer_index: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterType {
    FIR,
    IIR,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterParam {
    Gain,
    Delay,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidValue { param: FilterParam, value: f32 }
}

impl CombFilter {
    pub fn new(filter_type: FilterType, max_delay_secs: f32, sample_rate_hz: f32, num_channels: usize, gain: f32) -> Self {
        let delay_samples = (max_delay_secs * sample_rate_hz) as usize;
        let delay_lines = (0..num_channels)
            .map(|_| RingBuffer::<f32>::new(delay_samples))
            .collect();
        CombFilter {
            filter_type,
            delay_lines,
            gain,
            sample_rate: sample_rate_hz,
            buffer: vec![vec![0.0; delay_samples]; num_channels],
            buffer_index: 0,
        }
    }

    pub fn reset(&mut self) {
        for channel in &mut self.buffer {
            channel.fill(0.0);
        }
        self.buffer_index = 0;
    }

    pub fn process(&mut self, input: &[&[f32]], output: &mut [&mut [f32]]) {
        for (channel_index, &input_channel) in input.iter().enumerate() {
            let output_channel = &mut output[channel_index];
            let delay_line = &mut self.delay_lines[channel_index];

            for (sample_index, &input_sample) in input_channel.iter().enumerate() {
                let delayed_sample = delay_line.peek();


                let output_sample = input_sample + self.gain * delayed_sample;
                output_channel[sample_index] = output_sample;


                match self.filter_type {
                    FilterType::FIR => delay_line.push(input_sample),
                    FilterType::IIR => delay_line.push(output_sample),
                };
            }
        }
    }

    pub fn set_param(&mut self, param: FilterParam, value: f32) -> Result<(), Error> {
        match param {
            FilterParam::Gain => {
                if value >= 0.0 && value <= 1.0 {
                    self.gain = value;
                    Ok(())
                } else {
                    Err(Error::InvalidValue { param, value })
                }
            }
            FilterParam::Delay => {
                let delay_samples = (value * self.sample_rate) as usize;
                if delay_samples > 0 && delay_samples <= self.delay_lines[0].capacity() {
                    for delay_line in &mut self.delay_lines {
                        *delay_line = RingBuffer::new(delay_samples);
                    }
                    Ok(())
                } else {
                    Err(Error::InvalidValue { param, value })
                }
            }
        }
    }

    pub fn get_param(&self, param: FilterParam) -> f32 {
        match param {
            FilterParam::Gain => self.gain,
            FilterParam::Delay => {
                let delay_samples = self.delay_lines[0].capacity();
                delay_samples as f32 / self.sample_rate
            }
        }
    }
}
