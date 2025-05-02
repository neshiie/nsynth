use super::{Oscillator, Waveform};

#[derive(Clone)]
pub struct Voice {
    pub note: u8,
    pub osc: Oscillator,
    pub active: bool,
}

impl Voice {
    pub fn new(note: u8, sample_rate: f32, waveform: Waveform) -> Self {
        // standard method for converting midi note number to frequency in hz
        // 440.0: frequency of A4
        // 69.0 (nice): midi note number for A4
        // 12: num. of semitones in an octave
        let freq = 440.0 * 2.0f32.powf((note as f32 - 69.0) / 12.0);
        Self {
            note,
            active: true,
            osc: Oscillator {
                waveform,
                freq,
                sample_rate,
                phase: 0.0,
                gain: 0.0,
                target_gain: 1.0,
            },
        }
    }

    // call to Oscillator.next_sample
    pub fn next_sample(&mut self) -> f32 {
        self.osc.next_sample()
    }

    // note off: set active to false and set osc target gain to 0
    pub fn note_off(&mut self) {
        self.active = false;
        self.osc.target_gain = 0.0;
    }

    // returns true if gain is near zero and target gain is zero
    pub fn is_dead(&self) -> bool {
        self.osc.gain < 0.001 && self.osc.target_gain == 0.0
    }
}
