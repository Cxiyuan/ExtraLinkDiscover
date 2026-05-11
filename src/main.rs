#![cfg(windows)]
extern "system" {
    fn FreeConsole() -> i32;
}

mod app;
mod crawler;
mod filter;

fn main() {
    unsafe { FreeConsole(); }
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "ExtraLinkDiscover",
        options,
        Box::new(|cc| Ok(Box::new(app::ExtraLinkApp::new(cc)))),
    );
}
