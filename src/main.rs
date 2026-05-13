mod app;
mod crawler;
mod filter;

fn main() {
    // FreeConsole is only needed on Windows to hide the console window
    #[cfg(windows)]
    unsafe {
        extern "system" {
            fn FreeConsole() -> i32;
        }
        FreeConsole();
    }
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "外链爬取",
        options,
        Box::new(|cc| Ok(Box::new(app::ExtraLinkApp::new(cc)))),
    );
}
