//! Preset selection for this client, independent of a graphics backend.
//!
//! A preset is a user preference, not a claim of rendered retail parity.
//! Only implemented upgrades belong in `FINISHED_UPGRADES`; this initial
//! client has none. Adding an upgrade also requires an individual off switch
//! and a renderer capability check before exposing it in a player build.

use std::{error::Error, fmt, str::FromStr};

/// The complete set of presets currently accepted by the client.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Preset {
    Original,
    #[default]
    Forever,
}

impl Preset {
    /// Stable spelling for command-line arguments and future preferences.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Forever => "forever",
        }
    }
}

impl fmt::Display for Preset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reject unknown preferences instead of silently enabling a different preset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParsePresetError;

impl fmt::Display for ParsePresetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("expected preset 'original' or 'forever'")
    }
}

impl Error for ParsePresetError {}

impl FromStr for Preset {
    type Err = ParsePresetError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "original" => Ok(Self::Original),
            "forever" => Ok(Self::Forever),
            _ => Err(ParsePresetError),
        }
    }
}

/// Metadata for upgrades that are complete in this client.
///
/// This list intentionally contains no speculative settings or capability flags.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UpgradeInfo {
    pub id: &'static str,
    pub label: &'static str,
}

pub const FINISHED_UPGRADES: &[UpgradeInfo] = &[];

/// Session preferences; constructing or changing these never writes owner state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ClientOptions {
    preset: Preset,
}

impl ClientOptions {
    pub const fn new(preset: Preset) -> Self {
        Self { preset }
    }

    pub const fn preset(&self) -> Preset {
        self.preset
    }

    /// Switch the session's complete presentation preset in one operation.
    ///
    /// The caller applies the resolved choices to its renderer. There is no
    /// renderer or saved-preference I/O hidden in this method.
    pub fn set_preset(&mut self, preset: Preset) {
        self.preset = preset;
    }

    /// Finished upgrades selected by this preset, before device capability checks.
    ///
    /// Both presets currently resolve to an empty set. Future renderer-specific
    /// capability resolution must report requested and active choices separately.
    pub const fn enabled_upgrades(&self) -> &'static [UpgradeInfo] {
        match self.preset {
            Preset::Original => &[],
            Preset::Forever => FINISHED_UPGRADES,
        }
    }
}
