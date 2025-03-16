use eframe::egui;
use hpp_editor_v2::editor_app::EditorApp;

fn main() -> anyhow::Result<()> {
    color_backtrace::install();
    unsafe { std::env::set_var("RUST_LOG", "hats_plus_plus_editor=info,egui_glow=off,info") };
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
