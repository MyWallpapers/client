# Architecture Security Review — MyWallpaper Desktop

## Trust Boundary Analysis

### Boundary 1: Rust Backend <-> WebView Frontend
- **Status**: WEAK — withGlobalTauri:true gives remote page full IPC access
- **Risk**: Frontend compromise = full backend access
- **Recommendation**: Apply least-privilege permissions per capability

### Boundary 2: Frontend <-> Remote Server
- **Status**: MEDIUM — CSP enforced but overly permissive
- **Risk**: XSS via unsafe-inline, iframe from any HTTPS origin
- **Recommendation**: Tighten CSP (DONE), remove unsafe-inline when possible

### Boundary 3: App <-> Win32 APIs
- **Status**: GOOD — Platform code isolated with #[cfg(...)], unsafe minimized
- **Risk**: Pointer safety in COM interop (wry fork)
- **Recommendation**: Document pointer lifecycle, null-out on destroy

### Boundary 4: App <-> Updater
- **Status**: STRONG — minisign verification, endpoint locked to GitHub
- **Risk**: Downgrade attacks (FIXED with version comparison)
- **Recommendation**: Consider minimum version pinning

## Data Classification Matrix

| Data | Classification | Location | Protection |
|------|---------------|----------|------------|
| OAuth tokens | Sensitive | WebView memory | HTTPS only, validated URLs |
| Update signatures | Integrity-critical | GitHub releases | minisign verification |
| Deep-link URLs | Untrusted input | OS to Rust | Allowlist validation (FIXED) |
| System info | Internal | Init script | Exposed to frontend (low risk) |
| Log files | Operational | LOCALAPPDATA | Rotation with retention (FIXED) |
| Mouse coordinates | Transient | Atomic variables | Never persisted |
| COM pointers | Security-critical | Static atomics | Lifetime tied to app |

## IPC Permission Hardening Plan

### Current State (Overly Broad)
```json
"permissions": [
  "core:default",
  "core:event:default",
  "autostart:default",
  "updater:default",
  "opener:default",
  "deep-link:default",
  "log:default"
]
```

### Recommended State (Least Privilege)
Split into separate capabilities:
1. **Core capability** (always needed): core:default, core:event:default, log:default
2. **Update capability** (restricted): updater:check, updater:download (not updater:default)
3. **Auth capability** (restricted): opener:open-url (not opener:default)
4. **Startup capability** (restricted): autostart:enable, autostart:disable
5. **Deep-link capability**: deep-link:default (needed for OAuth callbacks)

## Zero-Trust Recommendations

1. **Validate all IPC inputs server-side** (in Rust) even if frontend validates them
2. **Minimize frontend trust** — frontend should be treated as untrusted
3. **Separate update logic** from frontend control — don't let frontend specify endpoints
4. **Sign and verify** the remote frontend hash if bundling isn't possible
5. **Log all security-relevant IPC calls** for audit trail

## Architectural Strengths
- Clean module separation (commands/commands_core/window_layer/tray)
- Platform code isolated with cfg guards
- Security validation in dedicated module (commands_core.rs)
- Comprehensive test coverage for security-critical paths (21 tests)
- Single-instance enforcement prevents process injection
- Atomic cleanup guarantees (ICONS_RESTORED flag)
