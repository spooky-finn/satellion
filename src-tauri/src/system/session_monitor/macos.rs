#![cfg(target_os = "macos")]

use std::ptr::NonNull;

use tauri::{AppHandle, Emitter};

use block2::StackBlock;
use objc2_foundation::{
    NSDistributedNotificationCenter, NSNotification, NSOperationQueue, NSString,
};

use crate::system::session_monitor::{SYS_SESSION_LOCKED_EVENT, SYS_SESSION_UNLOCKED_EVENT};

pub fn init(app: AppHandle) {
    unsafe {
        let center = NSDistributedNotificationCenter::defaultCenter();

        // Locked
        {
            let app = app.clone();

            let block = StackBlock::new(move |_notification: NonNull<NSNotification>| {
                let _ = app.emit(SYS_SESSION_LOCKED_EVENT, ());
            })
            .copy(); // IMPORTANT

            let name = NSString::from_str("com.apple.screenIsLocked");
            center.addObserverForName_object_queue_usingBlock(
                Some(&name),
                None,
                None::<&NSOperationQueue>,
                &block,
            );
        }

        // Unlocked
        {
            let app = app.clone();

            let block = StackBlock::new(move |_notification: NonNull<NSNotification>| {
                let _ = app.emit(SYS_SESSION_UNLOCKED_EVENT, ());
            })
            .copy(); // IMPORTANT

            let name = NSString::from_str("com.apple.screenIsUnlocked");
            center.addObserverForName_object_queue_usingBlock(
                Some(&name),
                None,
                None::<&NSOperationQueue>,
                &block,
            );
        }
    }
}
