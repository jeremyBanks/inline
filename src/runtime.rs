#[derive(Clone, Debug, Copy, Default)]
pub enum Mode {
    #[default]
    /// don't read the source files at all.
    Inactive,
    /// reads values from source files to verify that the current values
    /// round-trip successfully, but don't write anything.
    Verify,
    /// writes values back if they aren't equal to the expected value.
    Update,
}

impl Mode {
    pub fn read(self) -> bool {
        matches!(self, Mode::Verify | Mode::Update)
    }

    pub fn write(self) -> bool {
        matches!(self, Mode::Update)
    }

    pub fn inactive(self) -> bool {
        matches!(self, Mode::Inactive)
    }
}

pub static MODE: Lazy<Mode> = Lazy::new(|| {
    if env::var("LITTER_UPDATE").is_ok() {
        Mode::Update
    } else if env::var("LITTER_VERIFY").is_ok() {
        Mode::Verify
    } else {
        Mode::default()
    }
});
