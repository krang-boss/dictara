//! Appearance of the recording popup.
//!
//! The popup is the one piece of UI the user sees on every single dictation,
//! so its look is configurable rather than hard-coded. The laboratory screen
//! writes here; the popup window reads it.

use serde::{Deserialize, Serialize};

use crate::config::ConfigKey;

/// Variant ids from the oil and smoke concepts, which the cloud replaced.
///
/// A config written before the change still names one of these, and its
/// palette is tuned for a renderer that no longer exists — three opaque
/// pigments rather than a sky-to-cloud ramp. Carrying those numbers over would
/// hand the user a muddy orb they never chose, so the whole style is reset
/// instead. Can be dropped once no installed copy predates the change.
const LEGACY_VARIANTS: [&str; 11] = [
    "vapor-trail",
    "duotone",
    "marble",
    "lava",
    "aurora",
    "mercury",
    "filament",
    "ember",
    "ink",
    "plume",
    "toxic",
];

/// Everything the popup renderer needs to draw itself.
///
/// Values are normalized 0..1 unless noted, so the laboratory can expose them
/// as plain sliders and the shader can use them directly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PopupStyle {
    /// Id of the starting-point variant, kept so the UI can show what was picked
    /// even after the sliders have been nudged.
    pub variant: String,
    /// Palette as `#rrggbb` strings, read as a ramp from the densest sky to the
    /// lit cloud bank.
    pub palette: Vec<String>,
    /// How fast the motion evolves.
    pub speed: f32,
    /// How much the flow is distorted — wispy versus churning.
    pub turbulence: f32,
    /// Edge falloff: crisp disc versus diffuse cloud.
    pub softness: f32,
    /// How far colours are pushed from grey.
    #[serde(alias = "glow")]
    pub saturation: f32,
    /// How strongly the microphone level drives the motion.
    pub reactivity: f32,
    /// Rendered diameter in logical pixels.
    pub size: u32,
    /// Show the elapsed recording time under the orb.
    pub show_timer: bool,
}

impl Default for PopupStyle {
    fn default() -> Self {
        Self {
            variant: "periwinkle".to_string(),
            palette: vec![
                "#5c63fb".to_string(),
                "#7a8ffb".to_string(),
                "#b8c7f9".to_string(),
                "#e3ebfe".to_string(),
            ],
            speed: 0.38,
            turbulence: 0.5,
            softness: 0.08,
            saturation: 0.64,
            reactivity: 0.55,
            size: 200,
            show_timer: true,
        }
    }
}

impl PopupStyle {
    /// Clamp everything into range so a hand-edited config file cannot produce
    /// a popup that renders as a black square.
    pub fn sanitized(mut self) -> Self {
        if LEGACY_VARIANTS.contains(&self.variant.as_str()) {
            return Self::default();
        }

        self.speed = self.speed.clamp(0.0, 1.0);
        self.turbulence = self.turbulence.clamp(0.0, 1.0);
        self.softness = self.softness.clamp(0.0, 1.0);
        self.saturation = self.saturation.clamp(0.0, 1.0);
        self.reactivity = self.reactivity.clamp(0.0, 1.0);
        self.size = self.size.clamp(48, 320);

        self.palette.retain(|color| is_hex_color(color));
        // The ramp needs two ends to interpolate between.
        if self.palette.len() < 2 {
            self.palette = Self::default().palette;
        }
        self.palette.truncate(6);

        self
    }
}

fn is_hex_color(value: &str) -> bool {
    let Some(hex) = value.strip_prefix('#') else {
        return false;
    };
    matches!(hex.len(), 6 | 8) && hex.chars().all(|c| c.is_ascii_hexdigit())
}

impl ConfigKey<PopupStyle> {
    pub const POPUP_STYLE: Self = Self::new("popupStyle");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_clamps_out_of_range_values() {
        let style = PopupStyle {
            variant: "mint".to_string(),
            speed: 4.0,
            turbulence: -1.0,
            size: 5000,
            ..Default::default()
        }
        .sanitized();

        assert_eq!(style.speed, 1.0);
        assert_eq!(style.turbulence, 0.0);
        assert_eq!(style.size, 320);
    }

    #[test]
    fn sanitize_drops_invalid_colors_and_keeps_a_usable_palette() {
        let style = PopupStyle {
            palette: vec![
                "#ff0000".to_string(),
                "not a color".to_string(),
                "#00ff00".to_string(),
                "#0f0".to_string(),
            ],
            ..Default::default()
        }
        .sanitized();

        assert_eq!(
            style.palette,
            vec!["#ff0000".to_string(), "#00ff00".to_string()]
        );
    }

    #[test]
    fn sanitize_falls_back_when_too_few_colors_survive() {
        let style = PopupStyle {
            palette: vec!["#ff0000".to_string(), "red".to_string()],
            ..Default::default()
        }
        .sanitized();

        assert_eq!(style.palette, PopupStyle::default().palette);
    }

    #[test]
    fn a_style_from_a_removed_concept_is_reset_rather_than_carried_over() {
        let style = PopupStyle {
            variant: "lava".to_string(),
            palette: vec!["#ffd166".to_string(), "#7a1002".to_string()],
            size: 96,
            show_timer: false,
            ..Default::default()
        }
        .sanitized();

        assert_eq!(style, PopupStyle::default());
    }
}
