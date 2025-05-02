pub mod oscillator;
pub use oscillator::{Oscillator, Waveform};

pub mod voice;
pub use voice::Voice;

pub mod voice_manager;
pub use voice_manager::VoiceManager;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
