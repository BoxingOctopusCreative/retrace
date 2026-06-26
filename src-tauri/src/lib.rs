mod commands;
mod export;
mod setup;
mod state;
mod tracer;

use setup::installer;
use state::AppState;
use tauri::Emitter;
use tracer::vtracer::VtracerBackend;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            tracer: std::sync::Mutex::new(Box::new(VtracerBackend)),
            backend_statuses: std::sync::Mutex::new(installer::default_backend_statuses()),
        })
        .setup(|app| {
            use tauri::menu::{MenuBuilder, SubmenuBuilder};
            use tauri::Manager;

            // Replace defaults with real filesystem state so the UI reflects
            // any previously installed environment on first open.
            *app.state::<AppState>().backend_statuses.lock().unwrap() =
                installer::probe_backend_statuses(app.handle());

            // Apply window chrome settings that are unreliable via tauri.conf.json
            // when the CLI uses pre-built platform binaries.
            let win = app.get_webview_window("main").expect("no main window");
            #[cfg(target_os = "macos")]
            {
                win.set_title_bar_style(tauri::TitleBarStyle::Overlay)
                    .expect("failed to set title bar style");
            }
            #[cfg(not(target_os = "macos"))]
            {
                win.set_decorations(false)
                    .expect("failed to remove decorations");
            }

            let h = app.handle().clone();

            let edit = SubmenuBuilder::new(&h, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()?;

            #[cfg(target_os = "macos")]
            let menu = MenuBuilder::new(&h)
                .item(
                    &SubmenuBuilder::new(&h, "Re:Trace")
                        .text("about", "About Re:Trace")
                        .separator()
                        .quit()
                        .build()?,
                )
                .item(&edit)
                .build()?;

            #[cfg(not(target_os = "macos"))]
            let menu = MenuBuilder::new(&h)
                .item(&edit)
                .item(
                    &SubmenuBuilder::new(&h, "Help")
                        .text("about", "About Re:Trace")
                        .build()?,
                )
                .build()?;

            app.set_menu(menu)?;

            app.on_menu_event(|app_handle, event| {
                if event.id().as_ref() == "about" {
                    let _ = app_handle.emit("menu:about", ());
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::trace::trace_image,
            commands::trace::set_backend,
            commands::image::load_image_bytes,
            commands::export::export_vector,
            commands::setup::detect_gpu,
            commands::setup::get_disk_space,
            commands::setup::get_backend_statuses,
            commands::setup::get_python_env_installed,
            commands::setup::install_python_env,
            commands::setup::uninstall_python_env,
            commands::setup::download_model,
            commands::setup::cancel_download,
            commands::setup::uninstall_backend,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
