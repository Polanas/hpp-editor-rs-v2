pub mod animation_window;
pub mod animations;
pub mod console;
pub mod editor_app;
pub mod file_watcher;
pub mod files_watcher;
pub mod hats;
pub mod hats_data;
pub mod image;
pub mod shader;
pub mod tabs;
pub mod texture;
pub mod ui_text;

use editor_app::EditorApp;
use eframe::egui;

fn main() -> anyhow::Result<()> {
    color_backtrace::install();
    unsafe { std::env::set_var("RUST_LOG", "hats_plus_plus_editor=info,egui_glow=off,info") };
    flexi_logger::Logger::try_with_env()?.start()?;
    let native_opts = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default().with_inner_size((1600.0, 900.0)),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Editor",
        native_opts,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(EditorApp::new(cc)))
        }),
    );
    Ok(())
}
