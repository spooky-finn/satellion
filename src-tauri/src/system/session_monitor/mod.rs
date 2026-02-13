use tauri::AppHandle;

pub const SYS_SESSION_LOCKED_EVENT: &str = "system:session-locked";
pub const SYS_SESSION_UNLOCKED_EVENT: &str = "system:session-unlocked";

#[cfg(target_os = "macos")]
mod macos;

pub fn init(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    macos::init(app.clone());
}
