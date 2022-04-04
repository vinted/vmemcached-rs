const DEFAULT_BUFFER_SIZE: usize = 128;

/// Client settings
#[derive(Clone, Debug)]
pub struct Settings {
    /// Response buffer size
    pub buffer_size: usize,
}

impl Settings {
    /// Constructs a new `Settings`.
    ///
    /// Parameters are initialized with their default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set response buffer size
    pub fn buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;

        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            buffer_size: DEFAULT_BUFFER_SIZE,
        }
    }
}
