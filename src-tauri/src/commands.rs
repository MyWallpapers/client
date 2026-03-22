//! Tauri command handlers + business logic

use crate::error::{AppError, AppResult};
use crate::events::{AppEvent, EmitAppEvent};
use crate::system_monitor;
use log::info;
use serde::Serialize;
use std::sync::LazyLock;
use typeshare::typeshare;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Cached system info — OS details never change during app lifetime.
static CACHED_SYSTEM_INFO: LazyLock<SystemInfo> = LazyLock::new(|| SystemInfo {
    os: std::env::consts::OS.to_string(),
    os_version: os_info::get().version().to_string(),
    arch: std::env::consts::ARCH.to_string(),
    app_version: APP_VERSION.to_string(),
    tauri_version: tauri::VERSION.to_string(),
});
const GITHUB_RELEASE_DOWNLOAD_PATH: &str = "/MyWallpapers/client/releases/download/";
const GITHUB_RELEASE_LATEST_PATH: &str = "/MyWallpapers/client/releases/latest/download/";
const OAUTH_ALLOWED_HTTPS_HOSTS: &[&str] = &["accounts.google.com", "github.com"];

// ============================================================================
// Types
// ============================================================================

#[typeshare]
#[derive(Debug, Clone, Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub os_version: String,
    pub arch: String,
    pub app_version: String,
    pub tauri_version: String,
}

#[typeshare]
#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub body: Option<String>,
    pub date: Option<String>,
}

// ============================================================================
// Validation
// ============================================================================

/// Parse and validate the updater endpoint URL, returning the parsed URL on success.
fn validate_updater_endpoint(url: &str) -> AppResult<url::Url> {
    let parsed =
        url::Url::parse(url).map_err(|_| AppError::Validation("Invalid endpoint URL".into()))?;
    if parsed.scheme() != "https" {
        return Err(AppError::Validation("Endpoint must use HTTPS".into()));
    }
    if parsed.host_str() != Some("github.com") {
        return Err(AppError::Validation(
            "Endpoint must be on github.com".into(),
        ));
    }
    let path = parsed.path();
    if !path.starts_with(GITHUB_RELEASE_DOWNLOAD_PATH)
        && !path.starts_with(GITHUB_RELEASE_LATEST_PATH)
    {
        return Err(AppError::Validation(
            "Endpoint must point to MyWallpapers/client releases".into(),
        ));
    }
    Ok(parsed)
}

fn is_private_ipv4(ip: std::net::Ipv4Addr) -> bool {
    ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified()
}

fn is_allowed_oauth_https_host(host: &str) -> bool {
    host == "mywallpaper.online"
        || host.ends_with(".mywallpaper.online")
        || OAUTH_ALLOWED_HTTPS_HOSTS.contains(&host)
}

pub fn validate_oauth_url(url_str: &str) -> AppResult<()> {
    let parsed =
        url::Url::parse(url_str).map_err(|_| AppError::Validation("Invalid URL".into()))?;
    match parsed.scheme() {
        "https" => {}
        "http" => {
            let host = parsed.host_str().unwrap_or("");
            if host != "localhost" && host != "127.0.0.1" && host != "[::1]" {
                return Err(AppError::Validation(
                    "HTTP is only allowed for localhost".into(),
                ));
            }
            return Ok(());
        }
        _ => {
            return Err(AppError::Validation(
                "URL must use https:// (or http:// for localhost)".into(),
            ))
        }
    }
    match parsed.host() {
        Some(url::Host::Domain(host)) => {
            if !is_allowed_oauth_https_host(host) {
                return Err(AppError::Validation(
                    "URL host is not allowed for desktop OAuth".into(),
                ));
            }
        }
        Some(url::Host::Ipv4(ip)) => {
            if is_private_ipv4(ip) {
                return Err(AppError::Validation(
                    "HTTPS to private/internal IPs is not allowed".into(),
                ));
            }
        }
        Some(url::Host::Ipv6(ip)) => {
            if ip.is_loopback() || ip.is_unspecified() {
                return Err(AppError::Validation(
                    "HTTPS to private/internal IPs is not allowed".into(),
                ));
            }
            let s = ip.segments();
            if s[0] & 0xfe00 == 0xfc00 || s[0] & 0xffc0 == 0xfe80 {
                return Err(AppError::Validation(
                    "HTTPS to private/internal IPs is not allowed".into(),
                ));
            }
            if ip.to_ipv4_mapped().is_some_and(is_private_ipv4) {
                return Err(AppError::Validation(
                    "HTTPS to private/internal IPs is not allowed".into(),
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn parse_semver(v: &str) -> AppResult<(u32, u32, u32)> {
    let v = v.trim_start_matches('v');
    let v = v.split('-').next().unwrap_or(v);
    let p: Vec<&str> = v.split('.').collect();
    if p.len() != 3 {
        return Err(AppError::Validation(format!("Invalid version: {}", v)));
    }
    Ok((
        p[0].parse()
            .map_err(|_| AppError::Validation("bad major".into()))?,
        p[1].parse()
            .map_err(|_| AppError::Validation("bad minor".into()))?,
        p[2].parse()
            .map_err(|_| AppError::Validation("bad patch".into()))?,
    ))
}

fn validate_update_version(current: &str, candidate: &str) -> AppResult<()> {
    if parse_semver(candidate)? < parse_semver(current)? {
        return Err(AppError::Validation(format!(
            "Refusing downgrade from {} to {}",
            current, candidate
        )));
    }
    Ok(())
}

const ALLOWED_DEEP_LINK_ACTIONS: &[&str] = &["callback", "auth", "oauth", "login", "app"];

pub fn validate_deep_link(raw: &str) -> Option<String> {
    let parsed = url::Url::parse(raw).ok()?;
    if parsed.scheme() != "mywallpaper" {
        return None;
    }
    if let Some(host) = parsed.host_str() {
        if !host.is_empty() && !ALLOWED_DEEP_LINK_ACTIONS.contains(&host) {
            return None;
        }
    }
    Some(parsed.to_string())
}

// ============================================================================
// Commands
// ============================================================================

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    CACHED_SYSTEM_INFO.clone()
}

#[tauri::command]
pub fn get_system_data(categories: Vec<String>) -> system_monitor::SystemData {
    system_monitor::collect_system_data(system_monitor::parse_categories(&categories))
}

#[tauri::command]
pub fn subscribe_system_data(categories: Vec<String>) {
    system_monitor::set_poll_mask(system_monitor::parse_categories(&categories));
}

fn build_updater(
    app: &tauri::AppHandle,
    endpoint: Option<String>,
) -> AppResult<tauri_plugin_updater::Updater> {
    use tauri_plugin_updater::UpdaterExt;
    if let Some(url) = endpoint {
        let parsed = validate_updater_endpoint(&url)?;
        app.updater_builder()
            .endpoints(vec![parsed])
            .map_err(|e| AppError::Updater(format!("Invalid endpoint: {}", e)))?
            .build()
            .map_err(|e| AppError::Updater(format!("Build failed: {}", e)))
    } else {
        app.updater()
            .map_err(|e| AppError::Updater(format!("Updater not available: {}", e)))
    }
}

#[tauri::command]
pub async fn check_for_updates(
    app: tauri::AppHandle,
    endpoint: Option<String>,
) -> AppResult<Option<UpdateInfo>> {
    let updater = build_updater(&app, endpoint)?;
    match updater.check().await {
        Ok(Some(update)) => {
            validate_update_version(APP_VERSION, &update.version)?;
            info!("[updater] Update available: v{}", update.version);
            Ok(Some(UpdateInfo {
                version: update.version.clone(),
                current_version: APP_VERSION.to_string(),
                body: update.body.clone(),
                date: update.date.map(|d| d.to_string()),
            }))
        }
        Ok(None) => Ok(None),
        Err(e) => Err(AppError::Updater(format!("Update check failed: {}", e))),
    }
}

#[tauri::command]
pub async fn download_and_install_update(
    app: tauri::AppHandle,
    endpoint: Option<String>,
) -> AppResult<()> {
    let emit = |s: &str| {
        let _ = app.emit_app_event(&AppEvent::UpdateProgress {
            status: s.to_string(),
        });
    };
    emit("checking");
    let updater = build_updater(&app, endpoint)?;
    let update = updater
        .check()
        .await
        .map_err(|e| AppError::Updater(format!("Update check failed: {}", e)))?
        .ok_or_else(|| AppError::Updater("No update available".to_string()))?;
    validate_update_version(APP_VERSION, &update.version)?;
    emit("downloading");
    update
        .download_and_install(
            |_, _| {},
            || info!("[updater] Download complete, installing..."),
        )
        .await
        .map_err(|e| AppError::Updater(format!("Update install failed: {}", e)))?;
    emit("installed");
    Ok(())
}

#[tauri::command]
pub fn restart_app(app: tauri::AppHandle) {
    app.restart();
}

#[tauri::command]
pub fn open_oauth_in_browser(app: tauri::AppHandle, url: String) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;
    validate_oauth_url(&url)?;
    app.opener()
        .open_url(&url, None::<&str>)
        .map_err(|e| AppError::OAuth(format!("Failed to open browser: {}", e)))
}

#[tauri::command]
pub fn reload_window(app: tauri::AppHandle) -> AppResult<()> {
    app.emit_app_event(&AppEvent::ReloadApp)?;
    Ok(())
}

#[tauri::command]
pub fn get_media_info() -> AppResult<crate::media::MediaInfo> {
    crate::media::get_media_info()
}

#[tauri::command]
pub fn media_play_pause() -> AppResult<()> {
    crate::media::media_play_pause()
}

#[tauri::command]
pub fn media_next() -> AppResult<()> {
    crate::media::media_next()
}

#[tauri::command]
pub fn media_prev() -> AppResult<()> {
    crate::media::media_prev()
}

#[tauri::command]
pub fn open_path(app: tauri::AppHandle, path: String) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let parsed =
        url::Url::parse(&path).map_err(|_| AppError::Validation("Invalid URL".into()))?;
    match parsed.scheme() {
        "https" | "http" => app
            .opener()
            .open_url(&path, None::<&str>)
            .map_err(|e| AppError::Io(std::io::Error::other(format!("Failed to open: {}", e)))),
        _ => Err(AppError::Validation(
            "Only http/https URLs are allowed".into(),
        )),
    }
}

#[tauri::command]
pub fn update_discord_presence(details: String, state: String) -> AppResult<()> {
    crate::discord::update_presence(&details, &state)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- parse_semver --

    #[test]
    fn parse_semver_basic() {
        assert_eq!(parse_semver("1.2.3").unwrap(), (1, 2, 3));
    }

    #[test]
    fn parse_semver_with_v_prefix() {
        assert_eq!(parse_semver("v1.0.250").unwrap(), (1, 0, 250));
    }

    #[test]
    fn parse_semver_strips_prerelease() {
        assert_eq!(parse_semver("v1.0.250-dev").unwrap(), (1, 0, 250));
        assert_eq!(parse_semver("2.1.0-beta.1").unwrap(), (2, 1, 0));
    }

    #[test]
    fn parse_semver_rejects_short() {
        assert!(parse_semver("1.2").is_err());
        assert!(parse_semver("1").is_err());
    }

    #[test]
    fn parse_semver_rejects_non_numeric() {
        assert!(parse_semver("a.b.c").is_err());
        assert!(parse_semver("1.2.x").is_err());
    }

    // -- validate_update_version --

    #[test]
    fn validate_update_allows_upgrade() {
        assert!(validate_update_version("1.0.0", "1.0.1").is_ok());
        assert!(validate_update_version("1.0.0", "2.0.0").is_ok());
    }

    #[test]
    fn validate_update_allows_same_version() {
        assert!(validate_update_version("1.0.250", "1.0.250").is_ok());
    }

    #[test]
    fn validate_update_rejects_downgrade() {
        assert!(validate_update_version("1.0.250", "1.0.249").is_err());
        assert!(validate_update_version("2.0.0", "1.9.99").is_err());
    }

    // -- validate_oauth_url --

    #[test]
    fn oauth_allows_mywallpaper_https() {
        assert!(validate_oauth_url("https://mywallpaper.online/callback").is_ok());
        assert!(validate_oauth_url("https://app.mywallpaper.online/auth").is_ok());
    }

    #[test]
    fn oauth_allows_allowed_hosts() {
        assert!(validate_oauth_url("https://accounts.google.com/o/oauth2").is_ok());
        assert!(validate_oauth_url("https://github.com/login/oauth").is_ok());
    }

    #[test]
    fn oauth_allows_localhost_http() {
        assert!(validate_oauth_url("http://localhost:3000/callback").is_ok());
        assert!(validate_oauth_url("http://127.0.0.1:8080/auth").is_ok());
        assert!(validate_oauth_url("http://[::1]:3000/auth").is_ok());
    }

    #[test]
    fn oauth_rejects_http_non_localhost() {
        assert!(validate_oauth_url("http://evil.com/steal").is_err());
    }

    #[test]
    fn oauth_rejects_unknown_https_host() {
        assert!(validate_oauth_url("https://evil.com/phish").is_err());
    }

    #[test]
    fn oauth_rejects_private_ips() {
        assert!(validate_oauth_url("https://192.168.1.1/admin").is_err());
        assert!(validate_oauth_url("https://10.0.0.1/admin").is_err());
        assert!(validate_oauth_url("https://127.0.0.1/admin").is_err());
    }

    #[test]
    fn oauth_rejects_ipv6_private() {
        assert!(validate_oauth_url("https://[::1]/admin").is_err());
        assert!(validate_oauth_url("https://[fe80::1]/admin").is_err());
        assert!(validate_oauth_url("https://[fd12::1]/admin").is_err());
    }

    #[test]
    fn oauth_rejects_bad_schemes() {
        assert!(validate_oauth_url("ftp://mywallpaper.online/file").is_err());
        assert!(validate_oauth_url("javascript:alert(1)").is_err());
    }

    // -- validate_updater_endpoint --

    #[test]
    fn updater_allows_valid_release_url() {
        assert!(validate_updater_endpoint(
            "https://github.com/MyWallpapers/client/releases/latest/download/latest.json"
        )
        .is_ok());
    }

    #[test]
    fn updater_allows_download_path() {
        assert!(validate_updater_endpoint(
            "https://github.com/MyWallpapers/client/releases/download/v1.0.0/latest.json"
        )
        .is_ok());
    }

    #[test]
    fn updater_rejects_non_https() {
        assert!(validate_updater_endpoint(
            "http://github.com/MyWallpapers/client/releases/latest/download/latest.json"
        )
        .is_err());
    }

    #[test]
    fn updater_rejects_non_github() {
        assert!(validate_updater_endpoint("https://evil.com/latest.json").is_err());
    }

    #[test]
    fn updater_rejects_wrong_path() {
        assert!(validate_updater_endpoint(
            "https://github.com/other/repo/releases/latest/download/latest.json"
        )
        .is_err());
    }

    // -- validate_deep_link --

    #[test]
    fn deep_link_valid() {
        assert!(validate_deep_link("mywallpaper://callback?code=abc").is_some());
        assert!(validate_deep_link("mywallpaper://auth/token").is_some());
        assert!(validate_deep_link("mywallpaper://oauth").is_some());
        assert!(validate_deep_link("mywallpaper://login").is_some());
        assert!(validate_deep_link("mywallpaper://app").is_some());
    }

    #[test]
    fn deep_link_rejects_unknown_action() {
        assert!(validate_deep_link("mywallpaper://evil").is_none());
        assert!(validate_deep_link("mywallpaper://exec/cmd").is_none());
    }

    #[test]
    fn deep_link_rejects_wrong_scheme() {
        assert!(validate_deep_link("https://mywallpaper.online").is_none());
        assert!(validate_deep_link("not-a-url").is_none());
    }

    // -- parse_categories --

    #[test]
    fn parse_categories_basic() {
        use crate::system_monitor::*;
        let cats: Vec<String> = vec!["cpu".into(), "memory".into(), "uptime".into()];
        let mask = parse_categories(&cats);
        assert_eq!(mask, MASK_CPU | MASK_MEMORY | MASK_UPTIME);
    }

    #[test]
    fn parse_categories_unknown_ignored() {
        use crate::system_monitor::*;
        let cats: Vec<String> = vec!["cpu".into(), "unknown".into()];
        assert_eq!(parse_categories(&cats), MASK_CPU);
    }

    #[test]
    fn parse_categories_empty() {
        use crate::system_monitor::*;
        let cats: Vec<String> = vec![];
        assert_eq!(parse_categories(&cats), 0);
    }
}
