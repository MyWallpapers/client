# Development Guide

## Fast Iteration Loop (no CI/CD)

The Windows 11 VM is accessible via SSH for fast testing without CI/CD.

### VM Access
- **IP**: 192.168.122.150
- **User**: rayan
- **SSH key**: ~/.ssh/id_ed25519 (already configured)

### Build & Test on Windows VM

After making changes, pull and rebuild on the VM:

```bash
# Pull + build (incremental ~30-60s after first build)
ssh rayan@192.168.122.150 'cmd /c "cd C:\dev\client && git pull && cd src-tauri && cargo build 2>&1"'
```

### ⚠️ Windows Session 0 Limitation
Processes launched via SSH run in Session 0 (no GUI).
The app must be launched from inside the VM desktop session.
**Shortcut on VM desktop**: `C:\Users\rayan\Desktop\MyWallpaper-Dev.lnk`
Ask the user to double-click it after each build.

### Screenshot to verify behavior
```bash
DISPLAY=:0 XAUTHORITY=/run/user/1000/gdm/Xauthority import -window root /home/rayandu924/.openclaw/workspace/desktop-screenshot.png
```
Then analyze with vision AI to check the result without manual intervention.

### Typical Workflow
1. Fix code in `src-tauri/src/`
2. Commit + push
3. `ssh rayan@192.168.122.150 'cmd /c "cd C:\dev\client && git pull && cd src-tauri && cargo build 2>&1"'`
4. Ask user to double-click `MyWallpaper-Dev` on VM desktop
5. Screenshot + vision analysis
