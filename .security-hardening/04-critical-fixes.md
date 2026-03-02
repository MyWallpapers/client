# Critical Vulnerability Fixes Applied

## H-1 Mitigation: Least-Privilege IPC Capabilities
- Removed `api.mywallpaper.online` from IPC-capable remote URLs (API endpoint should never invoke desktop commands)
- Separated `opener` into its own capability file for isolated audit
- Only `dev.mywallpaper.online` and `app.mywallpaper.online` retain IPC access

## H-2 Fix: CSP Tightened
- `frame-src` restricted from `https:` (any) to specific mywallpaper.online origins
- Wildcard `*.mywallpaper.online` replaced with explicit subdomains throughout CSP
- `http://localhost:*` removed from script-src, style-src, default-src (kept in img-src/media-src for dev)

## M-1 Fix: DevTools Disabled in Production
- `devtools: false` in tauri.conf.json (was true for all builds)
- CI workflow re-enables for dev builds via sed before build step
- Prevents local IPC abuse via DevTools console

## M-2 Fix: Deep-Link Validation
- Added `validate_deep_link()` with action allowlist (callback, auth, oauth, login, app)
- URL parsed and normalized in Rust before forwarding to frontend
- Both entry points (single-instance args + deep-link://new-url) protected

## M-4 Fix: Updater Downgrade Prevention
- Added `validate_update_version()` with semver comparison
- Both `check_for_updates` and `download_and_install_update` reject older versions
- Handles v-prefix and -dev suffixes correctly

## Previously Applied (Phase 1)
- IPv6 SSRF validation (commands_core.rs)
- Null pointer guards in Win32 callbacks (window_layer.rs)
- 64-bit compile-time assertion (window_layer.rs)
- Consistent SeqCst atomic ordering (window_layer.rs)
- Path traversal protection with canonicalize (lib.rs)
- Error logging in cleanup operations (window_layer.rs)
- Named constants replacing magic numbers (window_layer.rs)
- Log rotation keeping last 5 files (lib.rs)
