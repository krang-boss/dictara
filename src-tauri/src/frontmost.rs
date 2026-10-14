//! Which application the user is currently typing into.
//!
//! Used to label history entries with where the text was pasted.

/// Bundle identifier and display name of the frontmost application.
#[derive(Debug, Clone, Default)]
pub struct FrontmostApp {
    pub bundle_id: Option<String>,
    pub name: Option<String>,
}

/// Look up the frontmost application, if the system will tell us.
///
/// Called from the recording thread just before pasting, while the target app
/// still has focus.
#[cfg(target_os = "macos")]
pub fn current() -> FrontmostApp {
    use objc2_app_kit::NSWorkspace;

    let Some(app) = NSWorkspace::sharedWorkspace().frontmostApplication() else {
        return FrontmostApp::default();
    };

    FrontmostApp {
        bundle_id: app.bundleIdentifier().map(|s| s.to_string()),
        name: app.localizedName().map(|s| s.to_string()),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn current() -> FrontmostApp {
    FrontmostApp::default()
}

/// Bring a specific application back to the front.
///
/// Used after the menu bar panel steals focus: the panel is our own window, so
/// pasting from it would otherwise send Cmd+V to a window the user was not
/// typing in. Returns whether the app was found and asked to activate.
#[cfg(target_os = "macos")]
pub fn activate(bundle_id: &str) -> bool {
    use objc2_app_kit::{NSApplicationActivationOptions, NSWorkspace};

    let apps = NSWorkspace::sharedWorkspace().runningApplications();

    for app in apps.iter() {
        let matches = app
            .bundleIdentifier()
            .is_some_and(|id| id.to_string() == bundle_id);

        if matches {
            return app.activateWithOptions(NSApplicationActivationOptions(0));
        }
    }

    false
}

#[cfg(not(target_os = "macos"))]
pub fn activate(_bundle_id: &str) -> bool {
    false
}
