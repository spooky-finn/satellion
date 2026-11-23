use rfd::{AsyncMessageDialog, MessageButtons, MessageDialogResult, MessageLevel};
use tauri::{AppHandle, Manager};
use tokio::sync::{Mutex, oneshot};

static ACTIVE_PROMPT: Mutex<()> = Mutex::const_new(());

pub fn visible_text(text: &str) -> String {
    let mut visible = String::new();
    for ch in text.chars() {
        match ch {
            '\n' | ' '..='~' => visible.push(ch),
            _ => visible.extend(ch.escape_default()),
        }
    }
    visible
}

pub async fn confirm(
    app: &AppHandle,
    title: &str,
    detail: &str,
    accept: &str,
    ensure_active: impl Fn() -> Result<(), String>,
) -> Result<(), String> {
    let _prompt = ACTIVE_PROMPT
        .try_lock()
        .map_err(|_| "Another wallet action is awaiting native confirmation")?;
    let window = app
        .get_webview_window("main")
        .ok_or("The wallet window is unavailable")?;
    let detail = visible_text(detail);
    let mut lines: Vec<String> = Vec::new();
    for line in detail.split('\n') {
        if line.is_empty() {
            lines.push(String::new());
        }
        lines.extend(line.as_bytes().chunks(72).map(|part| {
            // visible_text emits only ASCII.
            String::from_utf8(part.to_vec()).expect("ASCII dialog text")
        }));
    }
    let pages = lines.chunks(12);
    let total = pages.len();
    for (index, page) in pages.enumerate() {
        ensure_active()?;
        let last = index + 1 == total;
        let heading = if total == 1 {
            format!("Satellion · {title}")
        } else {
            format!("Satellion · {title} ({}/{total})", index + 1)
        };
        let (reply, answer) = oneshot::channel();
        let window = window.clone();
        let description = page.join("\n");
        // rfd 0.17.2 passes this text as a printf format string on GTK 3.
        #[cfg(any(
            target_os = "linux",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        let description = description.replace('%', "%%");
        let accept_label = if last { accept } else { "Next" }.to_owned();
        app.run_on_main_thread(move || {
            let response = AsyncMessageDialog::new()
                .set_title(heading)
                .set_description(description)
                .set_parent(&window)
                .set_level(MessageLevel::Warning)
                .set_buttons(MessageButtons::OkCancelCustom(
                    accept_label.clone(),
                    "Cancel".into(),
                ))
                .show();
            tauri::async_runtime::spawn(async move {
                let approved = match response.await {
                    MessageDialogResult::Ok => true,
                    MessageDialogResult::Custom(label) => label == accept_label,
                    _ => false,
                };
                let _ = reply.send(approved);
            });
        })
        .map_err(|_| "Could not open the native wallet confirmation")?;
        // Keep the prompt reserved until it closes, even if the OS locks meanwhile.
        if !answer.await.unwrap_or(false) {
            return Err("User rejected the wallet action".into());
        }
    }
    ensure_active()
}
