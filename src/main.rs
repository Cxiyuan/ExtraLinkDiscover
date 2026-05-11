mod app;
mod crawler;
mod filter;

fn main() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "ExtraLinkDiscover",
        options,
        Box::new(|_cc| Ok(Box::new(app::ExtraLinkApp::new()))),
    );
}
