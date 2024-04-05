use eframe::egui;
use simple_logger;

mod client;
mod session;
mod ui;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([300.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Xyncer",
        options,
        Box::new(|_cc| Box::<ui::Xyncer>::default()),
    )
}
