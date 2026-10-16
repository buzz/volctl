use std::path::Path;
use std::sync::{Mutex, Once};
use std::time::Duration;

use glib::prelude::*;
use gtk::gio::{self, FileMonitorEvent};
use gtk::prelude::*;

const FILE_NAME: &str = "osd.css";

static ONCE: Once = Once::new();
static RELOAD: Mutex<Option<glib::SourceId>> = Mutex::new(None);

/// Load `~/.config/volctl/osd.css` at user priority and reload it on change.
pub fn init() {
    ONCE.call_once(|| {
        let dir = glib::user_config_dir().join("volctl");
        let path = dir.join(FILE_NAME);

        let provider = gtk::CssProvider::new();
        let file_name = path.display().to_string();
        provider.connect_parsing_error(move |_, section, err| {
            tracing::warn!(file = %file_name, line = section.start_location().lines(), "osd.css: {err}");
        });
        sync_color_scheme(&provider);

        gtk::style_context_add_provider_for_display(
            gtk::gdk::Display::default().as_ref().expect("no display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );

        load(&provider, &path);
        watch(&dir, &provider, &path);
    });
}

fn load(provider: &gtk::CssProvider, path: &Path) {
    if path.is_file() {
        provider.load_from_path(path);
    } else {
        provider.load_from_string("");
    }
}

/// Debounce bursts of file events (editors emit several per save).
fn schedule_reload(provider: &gtk::CssProvider, path: &Path) {
    if let Some(pending) = RELOAD.lock().unwrap().take() {
        pending.remove();
    }
    let provider = provider.clone();
    let path = path.to_path_buf();
    *RELOAD.lock().unwrap() = Some(glib::timeout_add_local(
        Duration::from_millis(200),
        move || {
            *RELOAD.lock().unwrap() = None;
            load(&provider, &path);
            glib::ControlFlow::Break
        },
    ));
}

fn watch(dir: &Path, provider: &gtk::CssProvider, path: &Path) {
    let file = gio::File::for_path(dir);
    let Ok(monitor) = file.monitor_directory(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE)
    else {
        tracing::debug!("Failed to watch {}", dir.display());
        return;
    };
    let provider = provider.clone();
    let path = path.to_path_buf();
    monitor.connect_changed(move |_, file, other, event| {
        let is_target = |f: Option<&gio::File>| {
            f.and_then(|f| f.basename()).as_deref() == Some(std::path::Path::new(FILE_NAME))
        };
        if is_target(Some(file)) || (event == FileMonitorEvent::Renamed && is_target(other)) {
            schedule_reload(&provider, &path);
        }
    });
    // The monitor must outlive the app; it is a single inotify watch
    std::mem::forget(monitor);
}

/// The user file may use `@media (prefers-color-scheme: ...)`, which only
/// matches when the provider's property is in sync with the settings.
fn sync_color_scheme(provider: &gtk::CssProvider) {
    if glib::check_version(4, 20, 0).is_some() {
        return;
    }
    let Some(settings) = gtk::Settings::default() else {
        return;
    };
    provider.set_prefers_color_scheme(settings.gtk_interface_color_scheme());
    settings.connect_notify_local(
        Some("gtk-interface-color-scheme"),
        glib::clone!(
            #[strong]
            provider,
            #[strong]
            settings,
            move |_, _| {
                provider.set_prefers_color_scheme(settings.gtk_interface_color_scheme());
            }
        ),
    );
}
