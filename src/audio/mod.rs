//! Audio Engine — Per Architecture Appendix A (Audio Engine).
//!
//! Provides audio processing types and interfaces for speech-to-text,
//! text-to-speech, and audio streaming. Audio model execution delegates
//! to the AI Runtime when the `audio` feature is enabled.

/// Audio sample format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioFormat {
    /// 8-bit PCM.
    Pcm8,
    /// 16-bit PCM.
    Pcm16,
    /// 32-bit floating point.
    Float32,
    /// 64-bit floating point.
    Float64,
}

/// Audio channel configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioChannels {
    /// Mono (1 channel).
    Mono,
    /// Stereo (2 channels).
    Stereo,
}

/// Audio device for audio processing.
#[derive(Debug, Clone)]
pub struct AudioDevice {
    /// Device identifier.
    pub id: String,
    /// Human-readable device name.
    pub name: String,
    /// Whether the device is an input device.
    pub is_input: bool,
    /// Whether the device is an output device.
    pub is_output: bool,
    /// Supported sample rate in Hz.
    pub sample_rate_hz: u32,
}

/// Audio Manager — manages audio devices and processing.
pub struct AudioManager {
    devices: Vec<AudioDevice>,
    default_input: Option<String>,
    default_output: Option<String>,
}

impl AudioManager {
    /// Create a new audio manager with no devices.
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            default_input: None,
            default_output: None,
        }
    }

    /// Register an audio device.
    pub fn register_device(&mut self, device: AudioDevice) {
        if device.is_input && self.default_input.is_none() {
            self.default_input = Some(device.id.clone());
        }
        if device.is_output && self.default_output.is_none() {
            self.default_output = Some(device.id.clone());
        }
        self.devices.push(device);
    }

    /// List all registered device IDs.
    pub fn list_devices(&self) -> Vec<String> {
        self.devices.iter().map(|d| d.id.clone()).collect()
    }

    /// Get the default input device ID.
    pub fn default_input(&self) -> Option<&str> {
        self.default_input.as_deref()
    }

    /// Get the default output device ID.
    pub fn default_output(&self) -> Option<&str> {
        self.default_output.as_deref()
    }

    /// Get the number of registered devices.
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Get a device by ID.
    pub fn get_device(&self, id: &str) -> Option<&AudioDevice> {
        self.devices.iter().find(|d| d.id == id)
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}
