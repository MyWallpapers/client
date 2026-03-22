use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum AppEvent {
    WallpaperVisibility { visible: bool },
    UpdateProgress { status: String },
    SystemDataUpdate(Box<crate::system_monitor::SystemData>),
    DeepLink { url: String },
    ReloadApp,
}

impl AppEvent {
    pub fn event_name(&self) -> &'static str {
        match self {
            Self::WallpaperVisibility { .. } => "wallpaper-visibility",
            Self::UpdateProgress { .. } => "update-progress",
            Self::SystemDataUpdate(_) => "system-data-update",
            Self::DeepLink { .. } => "deep-link",
            Self::ReloadApp => "reload-app",
        }
    }
}

pub trait EmitAppEvent {
    fn emit_app_event(&self, event: &AppEvent) -> Result<(), tauri::Error>;
}

impl EmitAppEvent for tauri::AppHandle {
    fn emit_app_event(&self, event: &AppEvent) -> Result<(), tauri::Error> {
        use tauri::Emitter;
        // Emit raw inner data — frontend expects plain values, not the tagged enum wrapper.
        match event {
            AppEvent::WallpaperVisibility { visible } => {
                self.emit(event.event_name(), visible)
            }
            AppEvent::UpdateProgress { status } => {
                self.emit(event.event_name(), status)
            }
            AppEvent::SystemDataUpdate(data) => {
                self.emit(event.event_name(), data.as_ref())
            }
            AppEvent::DeepLink { url } => {
                self.emit(event.event_name(), url)
            }
            AppEvent::ReloadApp => {
                self.emit(event.event_name(), ())
            }
        }
    }
}
