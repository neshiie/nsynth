use super::{Voice, Waveform};
use std::collections::HashMap;

// we use a hashmap to store voices, using the midi note value as the key
// O(1) time for note_on and note_off
pub struct VoiceManager {
    pub voices: HashMap<u8, Voice>,
    pub sample_rate: f32,
    pub waveform: Waveform,
}

impl VoiceManager {
    pub fn new(sample_rate: f32, waveform: Waveform) -> Self {
        Self {
            voices: HashMap::new(),
            sample_rate,
            waveform,
        }
    }

    // inserts voice into the voices map
    pub fn note_on(&mut self, note: u8) {
        self.voices
            .insert(note, Voice::new(note, self.sample_rate, self.waveform));
    }

    // remove voice and turn off oscillator
    pub fn note_off(&mut self, note: u8) {
        if let Some(voice) = self.voices.get_mut(&note) {
            voice.note_off();
        }
    }

    pub fn mix(&mut self) -> f32 {
        let mut sum = 0.0; // keeps track of number of voice samples
        let mut active_count = 0; // keeps track of active voices

        // keep voices that aren't dead
        self.voices.retain(|_, voice| {
            let sample = voice.next_sample();
            if voice.osc.gain > 0.001 {
                sum += sample; // if sample is audible, add to sum
                active_count += 1; // and increment active counter
            }
            !voice.is_dead() // remove voice if it is dead (target gain = 0 and gain near 0)
        });

        // return average if active count is nonzero
        if active_count > 0 {
            sum / active_count as f32
        } else {
            0.0
        }
    }
}
