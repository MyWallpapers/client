//! Discord Rich Presence — shows "Using MyWallpaper" in Discord.
//! Retries connection periodically if Discord is not running at startup.

use crate::error::{AppError, AppResult};
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use log::{info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

// MyWallpaper Discord application ID (create at https://discord.com/developers/applications)
const DISCORD_APP_ID: &str = "1307092087033782272";
const RETRY_INTERVAL_SECS: u64 = 30;

static CLIENT: Mutex<Option<DiscordIpcClient>> = Mutex::new(None);
static RUNNING: AtomicBool = AtomicBool::new(false);

fn build_activity<'a>(details: &'a str, state: &'a str) -> activity::Activity<'a> {
    activity::Activity::new()
        .state(state)
        .details(details)
        .assets(
            activity::Assets::new()
                .large_image("logo")
                .large_text("MyWallpaper Desktop"),
        )
}

/// Try to connect to Discord RPC. Returns true on success.
fn try_connect() -> bool {
    match DiscordIpcClient::new(DISCORD_APP_ID) {
        Ok(mut client) => {
            if client.connect().is_ok() {
                let _ =
                    client.set_activity(build_activity("Using MyWallpaper", "Animated Wallpaper"));
                if let Ok(mut guard) = CLIENT.lock() {
                    *guard = Some(client);
                    info!("[discord] Rich Presence connected");
                    return true;
                }
            }
        }
        Err(e) => {
            warn!("[discord] Failed to create IPC client: {}", e);
        }
    }
    false
}

/// Connect to Discord RPC with periodic retry if Discord is not running.
pub fn init() {
    RUNNING.store(true, Ordering::Release);
    std::thread::spawn(|| {
        use std::time::Duration;

        if try_connect() {
            return;
        }
        warn!(
            "[discord] Discord not running, will retry every {}s",
            RETRY_INTERVAL_SECS
        );

        while RUNNING.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_secs(RETRY_INTERVAL_SECS));
            if !RUNNING.load(Ordering::Acquire) {
                break;
            }
            // Skip retry if already connected (another call to update_presence could have succeeded)
            if CLIENT.lock().is_ok_and(|g| g.is_some()) {
                return;
            }
            if try_connect() {
                return;
            }
        }
    });
}

/// Update the Discord Rich Presence activity.
pub fn update_presence(details: &str, state: &str) -> AppResult<()> {
    if let Ok(mut guard) = CLIENT.lock() {
        if let Some(ref mut client) = *guard {
            client
                .set_activity(build_activity(details, state))
                .map_err(|e| AppError::Discord(e.to_string()))?;
        }
    }
    Ok(())
}

/// Disconnect Discord RPC and stop retry thread.
pub fn shutdown() {
    RUNNING.store(false, Ordering::Release);
    if let Ok(mut guard) = CLIENT.lock() {
        if let Some(ref mut client) = *guard {
            let _ = client.close();
            info!("[discord] Rich Presence disconnected");
        }
        *guard = None;
    }
}
