# Frontend Security Hardening — MyWallpaper Desktop

## Note
The frontend is loaded remotely from dev.mywallpaper.online and is NOT part of this repository.
Frontend security is managed server-side. This document covers the Tauri-side protections.

## CSP Policy (Hardened)
```
default-src 'self' https://dev.mywallpaper.online https://app.mywallpaper.online https://api.mywallpaper.online;
img-src 'self' data: https: http://localhost:*;
media-src 'self' data: blob: https: http://localhost:*;
script-src 'self' 'unsafe-inline' https://dev.mywallpaper.online https://app.mywallpaper.online;
style-src 'self' 'unsafe-inline' https://dev.mywallpaper.online https://app.mywallpaper.online;
connect-src 'self' ipc: http://ipc.localhost https://dev.mywallpaper.online https://app.mywallpaper.online https://api.mywallpaper.online https://api.github.com wss://dev.mywallpaper.online wss://app.mywallpaper.online;
frame-src 'self' https://dev.mywallpaper.online https://app.mywallpaper.online;
```

### Changes Applied
1. frame-src: restricted from `https:` (any HTTPS) to specific origins
2. default-src: explicit subdomains instead of `*.mywallpaper.online` wildcard
3. script-src/style-src: removed `http://localhost:*` (kept in img/media for dev assets)
4. connect-src: removed wildcard, listed explicit subdomains

### Remaining Consideration
- `'unsafe-inline'` in script-src is required for the Tauri init script injection
- This cannot be removed without switching to nonce-based CSP, which requires Tauri framework changes
- Risk is mitigated by restricting which origins can serve scripts

## IPC Isolation
- api.mywallpaper.online removed from IPC-capable remote URLs
- opener isolated to separate capability for auditability
- DevTools disabled in production (prevents console IPC abuse)

## Recommendations for Remote Frontend
1. Implement SRI (Subresource Integrity) for all external resources
2. Use strict CSP with nonces if the frontend framework supports it
3. Validate `postMessage` origins for any cross-frame communication
4. Sanitize all deep-link URL parameters before DOM insertion
