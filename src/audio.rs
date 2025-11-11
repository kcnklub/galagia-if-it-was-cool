use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source, source::Buffered};
use std::fs::File;
use std::io::BufReader;

/// Audio manager for playing sound effects
pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    /// Pre-loaded and buffered fire sound (None if loading failed)
    fire_sound: Option<Buffered<Decoder<BufReader<File>>>>,
}

impl AudioManager {
    /// Create a new audio manager and pre-load audio files
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;

        // Pre-load and buffer the fire sound at startup
        let file = File::open("assests/sounds/flaunch.wav")?;
        let source = Decoder::new(BufReader::new(file))?;
        let fire_sound = Some(source.buffered());

        Ok(Self {
            _stream: stream,
            stream_handle,
            fire_sound,
        })
    }

    /// Play the weapon fire sound at default volume (30%)
    pub fn play_fire_sound(&self) {
        self.play_fire_sound_volume(0.01);
    }

    /// Play the weapon fire sound at a specific volume
    pub fn play_fire_sound_volume(&self, volume: f32) {
        // Only play if the sound was successfully loaded
        if let Some(fire_sound) = &self.fire_sound {
            // Ignore errors for sound playback - don't want to crash the game
            if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                sink.set_volume(volume);
                // Clone the buffered source (fast - just clones references)
                sink.append(fire_sound.clone());
                sink.detach();
            }
        }
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|err| {
            // Log error if audio initialization fails
            eprintln!("Warning: Failed to initialize audio: {}", err);
            eprintln!("Continuing without audio...");

            // Fallback: create audio manager without sound
            let (stream, stream_handle) =
                OutputStream::try_default().expect("Failed to create audio output stream");

            Self {
                _stream: stream,
                stream_handle,
                fire_sound: None, // No sound loaded - will silently fail to play
            }
        })
    }
}
