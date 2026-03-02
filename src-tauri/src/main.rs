// Prevents additional console window on Windows (debug & release)
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() {
    mywallpaper_desktop_lib::main();
}
