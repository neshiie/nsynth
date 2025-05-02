#[derive(Clone, Copy)]
pub enum Waveform {
    Sine,
    Square,
    Saw,
}

// oscillator struct definition
#[derive(Clone, Copy)]
pub struct Oscillator {
    pub waveform: Waveform,
    pub freq: f32,
    pub sample_rate: f32,
    pub phase: f32,
    pub gain: f32,
    pub target_gain: f32,
}

impl Oscillator {
    // setter for oscillator frequency
    // added smoothing to mitigate popping noise
    pub fn set_freq(&mut self, freq: f32) {
        if (self.freq - freq).abs() > f32::EPSILON {
            self.freq = freq;
            // Do NOT reset phase
        }
    }

    // grab next sample
    pub fn next_sample(&mut self) -> f32 {
        // standard logic for sin, square, saw waveforms
        let sample = match self.waveform {
            Waveform::Sine => (2.0 * std::f32::consts::PI * self.phase).sin(),
            Waveform::Square => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Waveform::Saw => 2.0 * self.phase - 1.0,
        };

        // approach target gain with exponential smoothing
        let smoothing = 0.001; // smaller = slower fade
        self.gain += (self.target_gain - self.gain) * smoothing;

        // increment phase by freq / rate, reset phase if > 1.0
        self.phase += self.freq / self.sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // set gain to 0 if gain is very very low
        if self.gain < 1e-5 {
            self.gain = 0.0;
        }

        // multiply sample by gain
        sample * self.gain
    }
}
