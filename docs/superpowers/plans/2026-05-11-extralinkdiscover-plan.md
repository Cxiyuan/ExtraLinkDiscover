# ExtraLinkDiscover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Windows GUI应用，爬取网站外部链接，支持并发控制、域名过滤、实时进度显示

**Architecture:**
- 单窗口GUI，使用eframe/egui构建
- 异步爬取使用tokio + reqwest + scraper
- 状态通过Channel实时同步到UI线程

**Tech Stack:** eframe, tokio, reqwest, scraper, open, serde, csv

---

## 文件结构

```
src/
├── main.rs          # 入口，初始化GUI
├── app.rs           # 应用状态、UI组件、事件处理
├── crawler.rs       # 异步爬虫逻辑
└── filter.rs        # 域名过滤逻辑
```

---

## Task 1: 项目依赖配置

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: 添加依赖**

```toml
[package]
name = "ExtraLinkDiscover"
version = "0.1.0"
edition = "2021"

[dependencies]
eframe = "0.29"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
scraper = "0.21"
open = "5"
csv = "1.3"
url = "2"

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["shell32"] }
```

- [ ] **Step 2: 提交**

```bash
git add Cargo.toml && git commit -m "chore: add dependencies"
```

---

## Task 2: 过滤模块

**Files:**
- Create: `src/filter.rs`
- Create: `tests/filter_test.rs`

- [ ] **Step 1: 写测试**

```rust
// tests/filter_test.rs
use ExtraLinkDiscover::filter::DomainFilter;

#[test]
fn test_filter_exact_domain() {
    let filter = DomainFilter::new("example.com");
    assert!(filter.should_filter("https://example.com/page"));
    assert!(filter.should_filter("https://www.example.com/page"));
    assert!(!filter.should_filter("https://other.com/page"));
}

#[test]
fn test_filter_subdomain() {
    let filter = DomainFilter::new("example.com");
    assert!(filter.should_filter("https://sub.example.com/page"));
    assert!(filter.should_filter("https://deep.sub.example.com/page"));
}

#[test]
fn test_filter_ip() {
    let filter = DomainFilter::new("192.168.1.1");
    assert!(filter.should_filter("https://192.168.1.1/page"));
}

#[test]
fn test_empty_filter() {
    let filter = DomainFilter::new("");
    assert!(!filter.should_filter("https://any.com/page"));
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test filter_test -- --nocapture`
Expected: FAIL - module not found

- [ ] **Step 3: 实现 filter.rs**

```rust
use url::Url;

pub struct DomainFilter {
    blocked_domains: Vec<String>,
}

impl DomainFilter {
    pub fn new(input: &str) -> Self {
        let domains: Vec<String> = input
            .lines()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        DomainFilter { blocked_domains: domains }
    }

    pub fn should_filter(&self, url: &str) -> bool {
        if self.blocked_domains.is_empty() {
            return false;
        }

        if let Ok(parsed) = Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                let host_lower = host.to_lowercase();
                for domain in &self.blocked_domains {
                    if host_lower == *domain || host_lower.ends_with(&format!(".{}", domain)) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn is_empty(&self) -> bool {
        self.blocked_domains.is_empty()
    }
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test filter_test -- --nocapture`
Expected: PASS

- [ ] **Step 5: 提交**

```bash
git add src/filter.rs tests/filter_test.rs && git commit -m "feat: add domain filter module"
```

---

## Task 3: 爬虫模块

**Files:**
- Create: `src/crawler.rs`
- Create: `tests/crawler_test.rs`

- [ ] **Step 1: 写测试**

```rust
// tests/crawler_test.rs
#[tokio::test]
async fn test_external_link_detection() {
    // Test HTML with internal and external links
    let html = r#"
        <html>
            <a href="https://example.com">External</a>
            <a href="/path">Internal</a>
        </html>
    "#;
    // ... test extraction logic
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test crawler -- --nocapture`
Expected: FAIL - module not found

- [ ] **Step 3: 实现 crawler.rs**

```rust
use reqwest::Client;
use scraper::{Html, Selector};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct Crawler {
    client: Client,
    filter: Arc<DomainFilter>,
    concurrency: usize,
}

#[derive(Debug, Clone)]
pub struct CrawlResult {
    pub external_url: String,
    pub source_url: String,
}

pub struct CrawlStats {
    pub pages_crawled: usize,
    pub links_found: usize,
}

impl Crawler {
    pub fn new(filter: DomainFilter, concurrency: usize) -> Self {
        let client = Client::builder()
            .user_agent("ExtraLinkDiscover/1.0")
            .build()
            .unwrap();
        Crawler {
            client,
            filter: Arc::new(filter),
            concurrency,
        }
    }

    pub async fn crawl(
        &self,
        start_url: &str,
        sender: mpsc::Sender<(CrawlResult, CrawlStats)>,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<(), String> {
        // BFS crawl with concurrency limit
        // For each page: fetch, parse links, send external links via channel
        todo!()
    }
}
```

- [ ] **Step 4: 实现完整的爬虫逻辑**

- [ ] **Step 5: 运行测试确认通过**

Run: `cargo test crawler -- --nocapture`
Expected: PASS

- [ ] **Step 6: 提交**

```bash
git add src/crawler.rs tests/crawler_test.rs && git commit -m "feat: add async crawler module"
```

---

## Task 4: 主应用模块 (UI)

**Files:**
- Create: `src/app.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: 实现 app.rs**

```rust
use eframe::egui;

pub struct ExtraLinkApp {
    pub url: String,
    pub concurrency: String,
    pub filter_domains: String,
    pub results: Vec<(String, String)>,  // (external_url, source_url)
    pub is_crawling: bool,
    pub stats: CrawlStats,
}

impl ExtraLinkApp {
    pub fn new() -> Self {
        ExtraLinkApp {
            url: String::new(),
            concurrency: "5".to_string(),
            filter_domains: String::new(),
            results: Vec::new(),
            is_crawling: false,
            stats: CrawlStats { pages_crawled: 0, links_found: 0 },
        }
    }
}

impl eframe::App for ExtraLinkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
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
                        ui.selectable_value(&mut self.concurrency, "5", "低 (5)");
                        ui.selectable_value(&mut self.concurrency, "10", "中 (10)");
                        ui.selectable_value(&mut self.concurrency, "20", "高 (20)");
                        ui.selectable_value(&mut self.concurrency, "50", "极高 (50)");
                    });
            });

            ui.horizontal(|ui| {
                if ui.button("开始爬取").clicked() {
                    if self.url.is_empty() {
                        // Show error
                    } else {
                        // Start crawling
                    }
                }
                if ui.button("停止").clicked() {
                    // Stop crawling
                }
            });

            // Results table
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (external, source) in &self.results {
                    ui.hyperlink_to(external, external);
                    ui.label(format!("  <-  {}", source));
                }
            });

            // Status bar
            ui.label(format!("已爬取: {}  已发现: {}",
                self.stats.pages_crawled, self.stats.links_found));
        });
    }
}
```

- [ ] **Step 2: 修改 main.rs**

```rust
mod app;
mod crawler;
mod filter;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "ExtraLinkDiscover",
        options,
        Box::new(|_cc| Ok(Box::new(app::ExtraLinkApp::new()))),
    );
}
```

- [ ] **Step 3: 测试编译**

Run: `cargo build`
Expected: 成功编译

- [ ] **Step 4: 提交**

```bash
git add src/app.rs src/main.rs && git commit -m "feat: add main app with UI"
```

---

## Task 5: CSV导出功能

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: 添加导出按钮和逻辑**

```rust
if ui.button("导出CSV").clicked() {
    if let Err(e) = self.export_csv() {
        // Show error
    }
}
```

- [ ] **Step 2: 实现导出函数**

```rust
fn export_csv(&self) -> Result<(), String> {
    let mut wtr = csv::Writer::from_path("results.csv").map_err(|e| e.to_string())?;
    wtr.write_record(&["外部链接", "所在页面"]).map_err(|e| e.to_string())?;
    for (url, source) in &self.results {
        wtr.write_record(&[url, source]).map_err(|e| e.to_string())?;
    }
    wtr.flush().map_err(|e| e.to_string())?;
    Ok(())
}
```

- [ ] **Step 3: 提交**

```bash
git add src/app.rs && git commit -m "feat: add CSV export"
```

---

## Task 6: 集成测试与调试

**Files:**
- 测试完整流程：输入URL → 爬取 → 显示结果 → 导出CSV

- [ ] **Step 1: 运行完整测试**

Run: `cargo run`
Expected: GUI窗口正常显示

- [ ] **Step 2: 验证错误处理**

- 空URL点击开始 → 显示错误提示
- 无效URL格式 → 显示错误提示

- [ ] **Step 3: 验证过滤功能**

- 输入过滤域名 → 确认过滤生效

- [ ] **Step 4: 验证导出功能**

- 点击导出 → 生成CSV文件

- [ ] **Step 5: 提交最终版本**

```bash
git add -A && git commit -m "feat: complete ExtraLinkDiscover"
```

---

## Task 7: 更新CI配置

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: 更新CI配置**

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build --verbose
      - name: Run tests
        run: cargo test --verbose
```

- [ ] **Step 2: 提交**

```bash
git add .github/workflows/ci.yml && git commit -m "ci: update CI for Windows build"
```