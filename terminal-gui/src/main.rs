mod app;
mod shell_path;
mod session;
mod process;
mod terminal;
mod terminal_view;

use app::TerminalApp;
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 500.0]),
            ..Default::default()
    };

    eframe::run_native(
        "Rust Terminal",
        options,
        Box::new(|cc| 
            Ok(Box::new(TerminalApp::new(cc.egui_ctx.clone())))),
    )
}