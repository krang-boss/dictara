mod credentials;
mod general;
mod presets;
pub mod shortcuts;
pub mod system;

// Re-export all commands
pub use credentials::*;
pub use general::*;
pub use presets::*;
pub use shortcuts::*;
pub use system::*;
