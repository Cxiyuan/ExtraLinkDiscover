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
        "外链爬取",
        options,
        Box::new(|cc| Ok(Box::new(app::ExtraLinkApp::new(cc)))),
    );
}
