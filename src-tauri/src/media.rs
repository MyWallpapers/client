//! Media playback info and control via WinRT (Windows only).

use crate::error::{AppError, AppResult};
use serde::Serialize;
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    /// "playing", "paused", "stopped", "unknown"
    pub playback_status: String,
    pub source_app: Option<String>,
}

#[cfg(target_os = "windows")]
fn get_manager(
) -> AppResult<windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager> {
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager;
    GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e| AppError::Media(format!("RequestAsync failed: {}", e)))?
        .get()
        .map_err(|e| AppError::Media(format!("Manager get failed: {}", e)))
}

#[cfg(target_os = "windows")]
fn winrt_string_opt(r: windows::core::Result<windows::core::HSTRING>) -> Option<String> {
    r.ok().map(|s| s.to_string()).filter(|s| !s.is_empty())
}

#[cfg(target_os = "windows")]
fn await_bool_op(
    name: &str,
    op: windows::core::Result<windows::Foundation::IAsyncOperation<bool>>,
) -> AppResult<()> {
    op.map_err(|e| AppError::Media(format!("{} failed: {}", name, e)))?
        .get()
        .map_err(|e| AppError::Media(format!("{} get failed: {}", name, e)))
        .and_then(|accepted| {
            if accepted {
                Ok(())
            } else {
                Err(AppError::Media(format!(
                    "{} was rejected by the active media session",
                    name
                )))
            }
        })
}

/// Get current media playback info from the system.
#[cfg(target_os = "windows")]
pub fn get_media_info() -> AppResult<MediaInfo> {
    let manager = get_manager()?;

    let session = match manager.GetCurrentSession() {
        Ok(s) => s,
        Err(_) => {
            return Ok(MediaInfo {
                playback_status: "stopped".into(),
                ..Default::default()
            })
        }
    };

    let status = session
        .GetPlaybackInfo()
        .ok()
        .and_then(|info| info.PlaybackStatus().ok())
        .map(|s| {
            use windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus;
            match s {
                GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing => "playing",
                GlobalSystemMediaTransportControlsSessionPlaybackStatus::Paused => "paused",
                GlobalSystemMediaTransportControlsSessionPlaybackStatus::Stopped => "stopped",
                GlobalSystemMediaTransportControlsSessionPlaybackStatus::Closed => "stopped",
                _ => "unknown",
            }
        })
        .unwrap_or("unknown")
        .to_string();

    let props = session
        .TryGetMediaPropertiesAsync()
        .ok()
        .and_then(|a| a.get().ok());

    let (title, artist, album) = match props {
        Some(ref p) => (
            winrt_string_opt(p.Title()),
            winrt_string_opt(p.Artist()),
            winrt_string_opt(p.AlbumTitle()),
        ),
        None => (None, None, None),
    };

    let source_app = winrt_string_opt(session.SourceAppUserModelId());

    Ok(MediaInfo {
        title,
        artist,
        album,
        playback_status: status,
        source_app,
    })
}

#[cfg(not(target_os = "windows"))]
pub fn get_media_info() -> AppResult<MediaInfo> {
    Ok(MediaInfo {
        playback_status: "unknown".into(),
        ..Default::default()
    })
}

/// Get the current active media session (manager → session).
#[cfg(target_os = "windows")]
fn current_session() -> AppResult<windows::Media::Control::GlobalSystemMediaTransportControlsSession>
{
    get_manager()?
        .GetCurrentSession()
        .map_err(|e| AppError::Media(format!("No active session: {}", e)))
}

/// Toggle play/pause on the current media session.
#[cfg(target_os = "windows")]
pub fn media_play_pause() -> AppResult<()> {
    await_bool_op(
        "TogglePlayPause",
        current_session()?.TryTogglePlayPauseAsync(),
    )
}

#[cfg(not(target_os = "windows"))]
pub fn media_play_pause() -> AppResult<()> {
    Err(AppError::Media("Not supported on this platform".into()))
}

/// Skip to next track.
#[cfg(target_os = "windows")]
pub fn media_next() -> AppResult<()> {
    await_bool_op("SkipNext", current_session()?.TrySkipNextAsync())
}

#[cfg(not(target_os = "windows"))]
pub fn media_next() -> AppResult<()> {
    Err(AppError::Media("Not supported on this platform".into()))
}

/// Skip to previous track.
#[cfg(target_os = "windows")]
pub fn media_prev() -> AppResult<()> {
    await_bool_op("SkipPrevious", current_session()?.TrySkipPreviousAsync())
}

#[cfg(not(target_os = "windows"))]
pub fn media_prev() -> AppResult<()> {
    Err(AppError::Media("Not supported on this platform".into()))
}
