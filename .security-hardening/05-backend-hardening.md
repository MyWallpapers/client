# Backend Security Hardening — MyWallpaper Desktop

## Rust Code Security Status

### Input Validation (Complete)
- All IPC command inputs validated in commands_core.rs
- OAuth URLs: scheme, host, IPv4/IPv6 private range checks
- Updater endpoints: HTTPS + github.com host + path prefix
- Deep-links: scheme + action allowlist
- Update versions: semver comparison prevents downgrades

### Error Handling (Clean)
- Zero `.unwrap()` calls in application code
- Single `.expect()` in app entry (acceptable, unrecoverable error)
- All cleanup operations log errors instead of silencing
- `.unwrap_or_default()` + `.is_invalid()` pattern for Win32 handles

### Memory Safety (Hardened)
- All Win32 enumeration callbacks have null pointer guards
- Compile-time 64-bit assertion prevents silent data loss
- Consistent atomic ordering (SeqCst for cross-thread, Relaxed for hot-path)
- Thread-safety documented for serialized WH_MOUSE_LL callbacks
- COM pointer lifecycle tied to application lifetime

### Unsafe Code (Minimized)
- All unsafe blocks are for Win32 API/MSAA (required, cannot avoid)
- Each block is appropriately scoped
- RAII patterns (SnapGuard) for resource cleanup
- Class name comparison is zero-allocation for hook performance

## Configuration Hardening
- DevTools: disabled in production, feature-flagged for dev
- CSP: explicit origins, no wildcards, frame-src restricted
- Capabilities: least-privilege, API domain removed from IPC
- Installer: currentUser mode (no elevation required)
- Updater: minisign signed, endpoint locked, downgrade prevented

## Secure Logging
- Log rotation: keeps last 5 files for forensics
- Path validated: canonicalize + starts_with prevents traversal
- No PII in log output (only system info, version, HWND addresses)
- Dual target: WebView + file logging

## Test Coverage
- 21 unit tests covering all security-critical paths
- Updater validation: 6 tests
- OAuth validation: 8 tests (including IPv6)
- Deep-link validation: 2 tests
- Version comparison: 4 tests
- System info: 1 test
