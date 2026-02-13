use tauri::AppHandle;

pub static SYS_SESSION_LOCKED_EVENT: &str = "system:session-locked";
pub static SYS_SESSION_UNLOCKED_EVENT: &str = "system:session-unlocked";

#[cfg(target_os = "macos")]
pub mod macos;

pub fn init(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    macos::init(app.clone());
}
