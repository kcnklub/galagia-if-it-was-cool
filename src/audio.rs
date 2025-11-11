use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

/// Audio manager for playing sound effects
pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    /// Shared sink for sound effects (currently unused but may be used for cleanup)
    #[allow(dead_code)]
    sfx_sinks: Arc<Mutex<Vec<Sink>>>,
}

impl AudioManager {
    /// Create a new audio manager
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;

        Ok(Self {
            _stream: stream,
            stream_handle,
            sfx_sinks: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Play a sound effect from a file path
    pub fn play_sound(
        &self,
        file_path: &str,
        volume: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Open the sound file
        let file = File::open(file_path)?;
        let source = Decoder::new(BufReader::new(file))?;

        // Create a new sink for this sound
        let sink = Sink::try_new(&self.stream_handle)?;

        // Set volume to 50%
        sink.set_volume(volume);

        // Append the sound to the sink and play
        sink.append(source);
        sink.detach();

        Ok(())
    }

    /// Play the weapon fire sound
    pub fn play_fire_sound(&self) {
        // Ignore errors for sound playback - don't want to crash the game
        let _ = self.play_sound("assests/sounds/flaunch.wav", 0.3);
    }

    pub fn play_fire_sound_volume(&self, volume: f32) {
        // Ignore errors for sound playback - don't want to crash the game
        let _ = self.play_sound("assests/sounds/flaunch.wav", volume);
    }

    /// Clean up finished sinks periodically
    #[allow(dead_code)]
    pub fn cleanup_finished_sinks(&self) {
        if let Ok(mut sinks) = self.sfx_sinks.lock() {
            sinks.retain(|sink| !sink.empty());
        }
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback: create a dummy audio manager if initialization fails
            let (stream, stream_handle) =
                OutputStream::try_default().expect("Failed to create audio output stream");
            Self {
                _stream: stream,
                stream_handle,
                sfx_sinks: Arc::new(Mutex::new(Vec::new())),
            }
        })
    }
}
