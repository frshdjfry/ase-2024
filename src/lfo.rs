// src/lfo.rs

use crate::ring_buffer::RingBuffer; // Assuming RingBuffer is implemented as before

pub struct LFO {
    wavetable: RingBuffer<f32>,
    phase_increment: f32,
    current_phase: f32,
    amplitude: f32,
    sample_rate: f32,
}

impl LFO {
    // Initializes a new LFO with given frequency, amplitude, and sample rate.
    pub fn new(frequency: f32, amplitude: f32, sample_rate: f32, wavetable_size: usize) -> Self {
        let mut wavetable = RingBuffer::<f32>::new(wavetable_size);
        for i in 0..wavetable_size {
            let phase = (i as f32 / wavetable_size as f32) * 2.0 * std::f32::consts::PI;
            wavetable.push(phase.sin());
        }

        let phase_increment = frequency / sample_rate;

        LFO {
            wavetable,
            phase_increment,
            current_phase: 0.0,
            amplitude,
            sample_rate,
        }
    }

    // Resets the LFO's phase to zero.
    pub fn reset(&mut self) {
        self.current_phase = 0.0;
    }

    // Sets the LFO's frequency and adjusts the phase increment accordingly.
    pub fn set_frequency(&mut self, frequency: f32) {
        self.phase_increment = frequency / self.sample_rate;
    }

    pub fn get_frequency(&self) -> f32 {
        self.phase_increment * self.sample_rate
    }
    // Sets the LFO's amplitude.
    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }

    // Processes the next sample, advancing the LFO's phase and returning the current value.
    pub fn tick(&mut self) -> f32 {
        let wavetable_size = self.wavetable.capacity();
        let index = (self.current_phase * wavetable_size as f32) as usize % wavetable_size;
        let value = self.wavetable.get(index) * self.amplitude;

        self.current_phase += self.phase_increment;
        if self.current_phase >= 1.0 {
            self.current_phase -= 1.0;
        }

        value
    }
}
