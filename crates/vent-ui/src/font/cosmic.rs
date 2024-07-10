use cosmic_text::FontSystem;

pub struct CosmicLoader {
    _font_system: cosmic_text::FontSystem,
}

impl Default for CosmicLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl CosmicLoader {
    pub fn new() -> Self {
        log::debug!(target: "ui", "initialising Cosmic-Text");
        let font_system = FontSystem::new();
        Self {
            _font_system: font_system,
        }
    }
}
