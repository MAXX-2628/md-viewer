#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

use slint::{ComponentHandle, ModelRc, VecModel};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

mod markdown;
mod settings;
mod zoom;

const SAMPLE: &str = "# Markdown syntax guide\n\n## Emphasis\n\n**Bold**, *italic*, and `inline code`.\n\n## Lists\n\n- First item\n- Second item\n  - Nested item\n\n## Table\n\n| Left | Right |\n| --- | ---: |\n| foo | bar |\n| baz | qux |\n\n## Quote\n\n> Markdown is a lightweight markup language.\n\n## Code\n\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n\n## Image\n\n![Markdown logo](https://example.com/markdown.png)\n\n[Learn more](https://example.com)\n";

fn show_source(window: &MainWindow, source: &str, label: &str, title: &str) {
    let blocks = markdown::to_blocks(source);
    window.set_blocks(ModelRc::new(VecModel::from(blocks)));
    window.set_word_count(markdown::word_count(source));
    window.set_status_text(label.into());
    window.set_document_title(title.into());
    window.set_has_document(true);
}

fn file_title(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn read_file(path: &Path) -> Result<String, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("Cannot open {}: {}", path.display(), error))?;
    Ok(text.trim_start_matches('\u{feff}').to_string())
}

fn open_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Markdown", &["md", "markdown", "txt"])
        .pick_file()
}

fn persist(state: &Rc<RefCell<settings::Settings>>, update: impl FnOnce(&mut settings::Settings)) {
    let mut settings = state.borrow_mut();
    update(&mut settings);
    settings.save();
}

fn main() -> Result<(), slint::PlatformError> {
    let window = MainWindow::new()?;
    let source = Rc::new(RefCell::new(String::new()));
    let state = Rc::new(RefCell::new(settings::Settings::load()));
    let current_path: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));

    {
        let settings = state.borrow();
        window.set_zoom(settings.zoom);
        window.set_dark(settings.dark);
    }

    // A file passed on the command line (double-click in Explorer) wins over the last session.
    let argument = std::env::args().nth(1).map(PathBuf::from);
    let startup = argument.clone().or_else(|| state.borrow().last_file.clone());
    match startup.map(|path| (read_file(&path), path)) {
        Some((Ok(text), path)) => {
            *source.borrow_mut() = text.clone();
            show_source(&window, &text, &path.display().to_string(), &file_title(&path));
            *current_path.borrow_mut() = Some(path.clone());
            if argument.is_some() {
                persist(&state, |settings| settings.last_file = Some(path));
            }
        }
        Some((Err(error), _)) if argument.is_some() => window.set_status_text(error.into()),
        _ => window.set_status_text("Open a .md file".into()),
    }

    let open = window.as_weak();
    let open_source = source.clone();
    let open_state = state.clone();
    let open_path = current_path.clone();
    window.on_open_file(move || {
        let Some(window) = open.upgrade() else {
            return;
        };
        let Some(path) = open_dialog() else {
            return;
        };
        match read_file(&path) {
            Ok(text) => {
                *open_source.borrow_mut() = text.clone();
                show_source(
                    &window,
                    &text,
                    &path.display().to_string(),
                    &file_title(&path),
                );
                *open_path.borrow_mut() = Some(path.clone());

                persist(&open_state, |settings| {
                    settings.last_file = Some(path);
                });
            }
            Err(error) => window.set_status_text(error.into()),
        }
    });

    let reload = window.as_weak();
    let reload_source = source.clone();
    let reload_path = current_path.clone();
    window.on_reload(move || {
        let Some(window) = reload.upgrade() else {
            return;
        };
        let Some(path) = reload_path.borrow().clone() else {
            window.set_status_text("No file to reload".into());
            return;
        };
        match read_file(&path) {
            Ok(text) => {
                *reload_source.borrow_mut() = text.clone();
                show_source(
                    &window,
                    &text,
                    &path.display().to_string(),
                    &file_title(&path),
                );
                window.set_status_text(format!("Reloaded {}", path.display()).into());
            }
            Err(error) => window.set_status_text(error.into()),
        }
    });

    let zoom_in = window.as_weak();
    let zoom_in_state = state.clone();
    window.on_zoom_in(move || {
        if let Some(window) = zoom_in.upgrade() {
            let value = zoom::step_in(window.get_zoom());
            window.set_zoom(value);
            persist(&zoom_in_state, |settings| settings.zoom = value);
        }
    });

    let zoom_out = window.as_weak();
    let zoom_out_state = state.clone();
    window.on_zoom_out(move || {
        if let Some(window) = zoom_out.upgrade() {
            let value = zoom::step_out(window.get_zoom());
            window.set_zoom(value);
            persist(&zoom_out_state, |settings| settings.zoom = value);
        }
    });

    let zoom_reset = window.as_weak();
    let zoom_reset_state = state.clone();
    window.on_zoom_reset(move || {
        if let Some(window) = zoom_reset.upgrade() {
            window.set_zoom(1.0);
            persist(&zoom_reset_state, |settings| settings.zoom = 1.0);
        }
    });

    let theme = window.as_weak();
    let theme_state = state.clone();
    window.on_toggle_theme(move || {
        if let Some(window) = theme.upgrade() {
            let dark = !window.get_dark();
            window.set_dark(dark);
            persist(&theme_state, |settings| settings.dark = dark);
        }
    });

    let copy = window.as_weak();
    let copy_source = source.clone();
    window.on_copy_source(move || {
        let text = copy_source.borrow().clone();
        let copied = arboard::Clipboard::new()
            .and_then(|mut clipboard| clipboard.set_text(text))
            .is_ok();
        if let Some(window) = copy.upgrade() {
            window.set_status_text(if copied {
                "Copied source".into()
            } else {
                "Clipboard unavailable".into()
            });
        }
    });

    let sample = window.as_weak();
    let sample_source = source.clone();
    let sample_state = state.clone();
    let sample_path = current_path.clone();
    window.on_load_sample(move || {
        if let Some(window) = sample.upgrade() {
            *sample_source.borrow_mut() = SAMPLE.to_string();
            show_source(&window, SAMPLE, "Sample document", "Sample document");
            *sample_path.borrow_mut() = None;
            persist(&sample_state, |settings| settings.last_file = None);
        }
    });

    let default = window.as_weak();
    window.on_set_default(move || {
        let opened = webbrowser::open("ms-settings:defaultapps").is_ok();
        if let Some(window) = default.upgrade() {
            window.set_status_text(if opened {
                "Choose MD Viewer in Windows Settings".into()
            } else {
                "Open Windows Settings to choose MD Viewer".into()
            });
        }
    });

    let minimize = window.as_weak();
    window.on_minimize_window(move || {
        if let Some(window) = minimize.upgrade() {
            window.window().set_minimized(true);
        }
    });

    window.on_close_window(|| {
        let _ = slint::quit_event_loop();
    });

    window.on_open_link(|link| {
        let _ = webbrowser::open(link.as_str());
    });

    window.run()?;
    Ok(())
}
