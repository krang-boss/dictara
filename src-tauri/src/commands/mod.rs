mod app;
mod history;
pub mod onboarding;
mod popup_style;
pub mod preferences;
mod recording;
pub mod registry;

// Re-export all commands for convenience
pub use app::*;
pub use history::*;
pub use onboarding::*;
pub use popup_style::*;
pub use preferences::*;
pub use recording::*;
