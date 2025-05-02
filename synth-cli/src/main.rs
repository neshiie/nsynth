// audio i/o and oscillator imports
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use synth_engine::VoiceManager;
use synth_engine::Waveform;

// midi imports
use midir::{Ignore, MidiInput};

// allows for shared data between threads
use std::sync::{Arc, Mutex};

fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("[X] No output device available");

    let supported_config = device.default_output_config()?;
    let mut config = supported_config.config();
    config.buffer_size = cpal::BufferSize::Fixed(512);
    let sample_rate = config.sample_rate.0 as f32;

    // create the shared voice manager
    let voices = Arc::new(Mutex::new(VoiceManager::new(sample_rate, Waveform::Sine)));

    // start midi listener
    midi_listen_thread(Arc::clone(&voices));

    // create audio stream using voice manager
    let stream = build_stream(&device, &config, Arc::clone(&voices))?;

    // start playback
    stream.play()?;
    println!("🎶 Polyphonic synth running... Press Ctrl+C to stop.");
    std::thread::sleep(std::time::Duration::from_secs(100));
    Ok(())
}

// construct the audio stream
fn build_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    voices: Arc<Mutex<VoiceManager>>,
) -> anyhow::Result<cpal::Stream> {
    // get num of output channels
    let channels = config.channels as usize;
    // clone the voice manager for use in the audio thread
    let voices_clone = Arc::clone(&voices);

    // building the output stream
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // iterate over audio frames, 1 sample per channel each frame
            for frame in data.chunks_mut(channels) {
                // mix active voices into one sample
                let mixed_sample = {
                    let mut voices = voices_clone.lock().unwrap();
                    voices.mix()
                };

                // write mixed sample to all channels in the frame
                for out in frame.iter_mut() {
                    *out = mixed_sample;
                }
            }
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn err_fn(err: cpal::StreamError) {
    eprintln!("Stream error: {}", err);
}

fn midi_listen_thread(voices: Arc<Mutex<VoiceManager>>) {
    let mut midi_in = MidiInput::new("midir-input").expect("Failed to create MIDI input");
    midi_in.ignore(Ignore::None);

    let in_ports = midi_in.ports();
    if in_ports.is_empty() {
        eprintln!("[X] No MIDI input devices found.");
        return;
    }

    // DEBUG: printing list of midi devices to console
    println!("Available MIDI input devices:");
    for (i, port) in in_ports.iter().enumerate() {
        let name = midi_in
            .port_name(port)
            .unwrap_or_else(|_| "<unknown>".into());
        println!("  [{}] {}", i, name);
    }

    if in_ports.len() <= 1 {
        eprintln!(
            "[X] MIDI device #1 not available. Found only {} ports.",
            in_ports.len()
        );
        return;
    }

    // FIXME: hardcoding use of midi device #1 for now
    // make this selectable in the GUI later
    let in_port = &in_ports[1];
    let port_name = midi_in
        .port_name(in_port)
        .unwrap_or_else(|_| "<unknown>".into());
    println!("Connecting to: {}", port_name);

    let voices_clone = Arc::clone(&voices);

    // connect to in_port
    match midi_in.connect(
        in_port,
        "midir-read-input",
        move |_, message, voices| {
            // status, note, velocity each correspond to one byte in midi input
            if message.len() == 3 {
                let status = message[0] & 0xF0;
                let note = message[1];
                let velocity = message[2];

                // lock voice manager mutex
                let mut voices = voices.lock().unwrap();

                // handle note when velocity != 0
                if status == 0x90 && velocity > 0 {
                    println!("+++ Note ON: {} (vel {})", note, velocity);
                    voices.note_on(note);
                }
                // handle note off functionality
                else if status == 0x80 || (status == 0x90 && velocity == 0) {
                    println!("--- Note OFF: {}", note);
                    voices.note_off(note);
                }
            }
        },
        voices_clone,
    ) {
        Ok(conn) => {
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                #[allow(unreachable_code)]
                {
                    drop(conn);
                }
            });
        }
        Err(e) => {
            eprintln!("MIDI connection failed: {}", e);
        }
    }
}
