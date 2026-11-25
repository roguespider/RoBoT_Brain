//! Vision Engine — Per Architecture Appendix A (Vision Engine).
//!
//! Provides vision processing types and interfaces for image loading,
//! OCR, object detection, and scene analysis. Vision model execution
//! delegates to the AI Runtime when the `audio` feature is enabled.

/// Image format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageFormat {
    /// PNG image.
    Png,
    /// JPEG image.
    Jpeg,
    /// BMP image.
    Bmp,
    /// TIFF image.
    Tiff,
    /// WebP image.
    WebP,
}

/// Image dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageDimensions {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

/// Vision device for vision processing.
#[derive(Debug, Clone)]
pub struct VisionDevice {
    /// Device identifier.
    pub id: String,
    /// Human-readable device name.
    pub name: String,
    /// Whether the device supports GPU acceleration.
    pub gpu_accelerated: bool,
    /// Maximum image width supported.
    pub max_width: u32,
    /// Maximum image height supported.
    pub max_height: u32,
}

/// Vision Manager — manages vision devices and processing.
pub struct VisionManager {
    devices: Vec<VisionDevice>,
    default_device: Option<String>,
}

impl VisionManager {
    /// Create a new vision manager with no devices.
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            default_device: None,
        }
    }

    /// Register a vision device.
    pub fn register_device(&mut self, device: VisionDevice) {
        if self.default_device.is_none() {
            self.default_device = Some(device.id.clone());
        }
        self.devices.push(device);
    }

    /// List all registered device IDs.
    pub fn list_devices(&self) -> Vec<String> {
        self.devices.iter().map(|d| d.id.clone()).collect()
    }

    /// Get the default device ID.
    pub fn default_device(&self) -> Option<&str> {
        self.default_device.as_deref()
    }

    /// Get the number of registered devices.
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Get a device by ID.
    pub fn get_device(&self, id: &str) -> Option<&VisionDevice> {
        self.devices.iter().find(|d| d.id == id)
    }
}

impl Default for VisionManager {
    fn default() -> Self {
        Self::new()
    }
}
