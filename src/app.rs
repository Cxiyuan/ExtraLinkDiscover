use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::crawler::{CrawlResult, Crawler, CrawlStats};
use crate::filter::DomainFilter;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;

pub struct ExtraLinkApp {
    pub url: String,
    pub concurrency: String,
    pub filter_domains: String,
    pub results: Vec<(String, String)>,
    pub is_crawling: bool,
    pub stats: CrawlStats,
    pub error_message: Option<String>,
    pub stop_flag: Option<Arc<AtomicBool>>,
    pub crawl_handle: Option<thread::JoinHandle<()>>,
    pub result_receiver: Option<Arc<Mutex<mpsc::Receiver<(CrawlResult, CrawlStats)>>>>,
    pub show_debug: bool,
    pub current_crawl_url: String,
    /// Maximum results to display in UI (prevents lag from huge result sets)
    pub max_displayed_results: usize,
}

impl ExtraLinkApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Configure Chinese font - try to load from system font directory
        let mut fonts = egui::FontDefinitions::default();

        // Try to load simsun font from Windows fonts directory
        let font_path = std::path::PathBuf::from("C:\\Windows\\Fonts\\simsun.ttc");
        if let Ok(font_data) = std::fs::read(&font_path) {
            fonts.font_data.insert(
                "simsun".to_owned(),
                egui::FontData::from_owned(font_data),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "simsun".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("simsun".to_owned());
        }

        cc.egui_ctx.set_fonts(fonts);

        ExtraLinkApp {
            url: String::new(),
            concurrency: "5".to_string(),
            filter_domains: String::new(),
            results: Vec::new(),
            is_crawling: false,
            stats: CrawlStats { pages_crawled: 0, links_found: 0, current_url: String::new() },
            error_message: None,
            stop_flag: None,
            crawl_handle: None,
            result_receiver: None,
            show_debug: false,
            current_crawl_url: String::new(),
            max_displayed_results: 1000,
        }
    }

    pub fn start_crawl(&mut self) {
        if self.url.is_empty() {
            self.error_message = Some("请输入目标网站URL".to_string());
            return;
        }

        let start_url = self.url.trim().to_string();
        let concurrency: usize = self.concurrency.parse().unwrap_or(5);
        let filter_domains = self.filter_domains.clone();
        let stop_flag = Arc::new(AtomicBool::new(false));
        self.stop_flag = Some(stop_flag.clone());

        self.results.clear();
        self.stats = CrawlStats { pages_crawled: 0, links_found: 0, current_url: String::new() };
        self.is_crawling = true;
        self.error_message = None;
        self.current_crawl_url = start_url.clone();

        // Spawn a thread with a tokio runtime
        let (tx, rx) = mpsc::channel::<(CrawlResult, CrawlStats)>(1000);
        let result_receiver = Arc::new(Mutex::new(rx));

        let filter = if filter_domains.is_empty() {
            DomainFilter::new("")
        } else {
            DomainFilter::new(&filter_domains)
        };

        let crawler = Crawler::new(filter, concurrency);
        let stop_flag_clone = stop_flag.clone();

        self.crawl_handle = Some(thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                crawler.crawl(&start_url, tx, stop_flag_clone).await;
            });
        }));

        self.result_receiver = Some(result_receiver);
    }

    pub fn stop_crawl(&mut self) {
        if let Some(stop_flag) = &self.stop_flag {
            stop_flag.store(true, Ordering::Relaxed);
        }
        self.is_crawling = false;
        // Clean up receiver and handle to prevent resource leaks
        self.result_receiver = None;
        self.crawl_handle = None;
    }

    pub fn export_csv(&self) -> Result<(), String> {
        let mut wtr = csv::Writer::from_path("results.csv").map_err(|e| e.to_string())?;
        wtr.write_record(&["外部链接", "所在页面"]).map_err(|e| e.to_string())?;
        for (url, source) in &self.results {
            wtr.write_record(&[url, source]).map_err(|e| e.to_string())?;
        }
        wtr.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clear_results(&mut self) {
        self.results.clear();
        self.stats = CrawlStats { pages_crawled: 0, links_found: 0, current_url: String::new() };
        self.error_message = None;
    }

    pub fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
    }

    fn poll_results(&mut self) {
        // Limit how many results we process per frame to prevent UI stutter
        const MAX_RESULTS_PER_FRAME: usize = 50;

        if let Some(ref receiver) = self.result_receiver {
            if let Ok(mut rx) = receiver.lock() {
                let mut processed = 0;
                loop {
                    if processed >= MAX_RESULTS_PER_FRAME {
                        // Only request repaint if we hit the limit (more results pending)
                        return;
                    }
                    match rx.try_recv() {
                        Ok((result, stats)) => {
                            // Skip placeholder results (in-progress updates)
                            if !result.external_url.is_empty() {
                                self.results.push((result.external_url, result.source_url));
                            }
                            self.stats = stats;
                            processed += 1;
                        }
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => break,
                    }
                }
            }
        }

        // Check if crawl is finished
        if let Some(handle) = self.crawl_handle.take() {
            if handle.is_finished() {
                self.is_crawling = false;
            } else {
                self.crawl_handle = Some(handle);
            }
        }
    }
}

impl eframe::App for ExtraLinkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll for results if crawling
        if self.is_crawling {
            self.poll_results();
            // Throttle repaints to ~30fps (33ms) to avoid UI stutter
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }

        // Top panel for inputs
        egui::TopBottomPanel::top(egui::Id::new("top_panel"))
            .show(ctx, |ui| {
            // Error message area
            if let Some(ref err) = self.error_message {
                ui.colored_label(egui::Color32::RED, err);
            }

            // Input section
            ui.horizontal(|ui| {
                ui.label("目标网站:");
                ui.text_edit_singleline(&mut self.url);
            });

            ui.horizontal(|ui| {
                ui.label("并发级别:");
                egui::ComboBox::from_id_salt("concurrency")
                    .selected_text(&self.concurrency)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.concurrency, "5".to_string(), "低 (5)");
                        ui.selectable_value(&mut self.concurrency, "10".to_string(), "中 (10)");
                        ui.selectable_value(&mut self.concurrency, "20".to_string(), "高 (20)");
                        ui.selectable_value(&mut self.concurrency, "50".to_string(), "极高 (50)");
                    });
            });

            ui.horizontal(|ui| {
                ui.label("过滤域名:");
                ui.text_edit_singleline(&mut self.filter_domains);
                ui.label("(可选，逗号分隔，支持*.domain格式)");
            });

            ui.horizontal(|ui| {
                let start_button = ui.add_enabled(!self.is_crawling, egui::Button::new("开始爬取"));
                if start_button.clicked() {
                    self.start_crawl();
                }

                let stop_button = ui.add_enabled(self.is_crawling, egui::Button::new("停止"));
                if stop_button.clicked() {
                    self.stop_crawl();
                }

                if ui.button("清除").clicked() {
                    self.clear_results();
                }

                ui.separator();

                if ui.button("导出CSV").clicked() {
                    if let Err(e) = self.export_csv() {
                        self.error_message = Some(format!("导出失败: {}", e));
                    }
                }

                let debug_label = if self.show_debug { "关闭调试" } else { "调试" };
                if ui.button(debug_label).clicked() {
                    self.toggle_debug();
                }
            });

            // Debug window
            if self.show_debug {
                ui.separator();
                ui.label("调试信息:");
                ui.label(format!("URL: {}", self.url));
                ui.label(format!("并发级别: {}", self.concurrency));
                ui.label(format!("过滤域名: {}", self.filter_domains));
                ui.label(format!("结果数量: {}", self.results.len()));
                ui.label(format!("爬取状态: {}", if self.is_crawling { "爬取中" } else { "已停止" }));
                if let Some(handle) = &self.crawl_handle {
                    ui.label(format!("线程运行中: {}", !handle.is_finished()));
                }
            }
        });

        // Central area for results
        egui::CentralPanel::default().show(ctx, |ui| {
            // Results header
            ui.horizontal(|ui| {
                ui.label("外部链接");
                ui.label("          所在页面");
            });

            // Show warning if results are limited
            let total_results = self.results.len();
            let displayed_results = self.results.iter().rev().take(self.max_displayed_results).count();
            if total_results > self.max_displayed_results {
                ui.label(egui::Color32::YELLOW, format!(
                    "显示最新 {} / {} 条结果 (结果过多，已限制显示)",
                    displayed_results, total_results
                ));
            }
            ui.separator();

            // Results table - use show_rows for efficient virtualized rendering
            // This only renders visible rows, preventing lag with large result sets
            let row_height = 24.0;
            let available_height = ui.available_height();
            let num_visible_rows = (available_height / row_height).ceil() as usize;
            let total_rows = std::cmp::min(total_results, self.max_displayed_results);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, row_height, total_rows, |ui, row_range| {
                    // Convert row indices to result indices (newest first)
                    for row in row_range {
                        let result_idx = total_results - 1 - row;
                        if let Some((external, source)) = self.results.get(result_idx) {
                            ui.horizontal(|ui| {
                                // Use selectable_label for external URL (shows full URL on hover)
                                let ext_short = if external.len() > 60 {
                                    format!("{}...", &external[..60])
                                } else {
                                    external.clone()
                                };
                                let ext_response = ui.selectable_label(false, ext_short);
                                if ext_response.clicked() {
                                    let _ = open::that(external);
                                }
                                ui.label("<-");
                                let src_short = if source.len() > 60 {
                                    format!("{}...", &source[..60])
                                } else {
                                    source.clone()
                                };
                                let src_response = ui.selectable_label(false, src_short);
                                if src_response.clicked() {
                                    let _ = open::that(source);
                                }
                            });
                        }
                    }
                });
        });

        // Bottom panel for status
        egui::TopBottomPanel::bottom(egui::Id::new("bottom_panel"))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let status = if self.is_crawling { "爬取中" } else { "已停止" };
                    ui.label(format!("状态: {}  已爬取: {}  已发现: {}",
                        status, self.stats.pages_crawled, self.stats.links_found));
                    if self.is_crawling {
                        ui.separator();
                        ui.label(format!("正在爬取: {}", self.current_crawl_url));
                    }
                });
            });
    }
}