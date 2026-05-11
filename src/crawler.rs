use crate::filter::DomainFilter;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use url::Url;

#[derive(Debug, Clone)]
pub struct CrawlResult {
    pub external_url: String,
    pub source_url: String,
}

#[derive(Debug)]
pub struct CrawlStats {
    pub pages_crawled: usize,
    pub links_found: usize,
    pub current_url: String,
}

impl CrawlStats {
    pub fn new() -> Self {
        CrawlStats {
            pages_crawled: 0,
            links_found: 0,
            current_url: String::new(),
        }
    }
}

impl Default for CrawlStats {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Crawler {
    client: Client,
    filter: Arc<DomainFilter>,
    concurrency: usize,
}

impl Crawler {
    pub fn new(filter: DomainFilter, concurrency: usize) -> Self {
        let client = Client::builder()
            .user_agent("ExtraLinkDiscover/1.0")
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        Crawler {
            client,
            filter: Arc::new(filter),
            concurrency,
        }
    }

    pub fn concurrency(&self) -> usize {
        self.concurrency
    }

    pub async fn crawl(
        &self,
        start_url: &str,
        sender: mpsc::Sender<(CrawlResult, CrawlStats)>,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<(), String> {
        let start_parsed = Url::parse(start_url)
            .map_err(|e| format!("Failed to parse start URL: {}", e))?;

        let base_domain = start_parsed
            .host_str()
            .ok_or("Start URL has no host")?
            .to_lowercase();

        let mut visited = std::collections::HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_url.to_string());

        let mut pages_crawled = 0usize;
        let mut links_found = 0usize;

        while !queue.is_empty() && !stop_flag.load(Ordering::Relaxed) {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            // Process URLs up to concurrency limit
            let max_process = std::cmp::min(queue.len(), self.concurrency);
            let mut handles = Vec::new();
            let mut urls_processed = Vec::with_capacity(max_process);

            for _ in 0..max_process {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }
                let url = match queue.pop_front() {
                    Some(u) => u,
                    None => break,
                };

                if visited.contains(&url) {
                    continue;
                }
                visited.insert(url.clone());

                let client = self.client.clone();
                let filter = self.filter.clone();
                let base_domain = base_domain.clone();
                let stop_flag = stop_flag.clone();
                let url_for_processing = url.clone();
                let url_for_parser = url.clone();

                let handle = tokio::spawn(async move {
                    if stop_flag.load(Ordering::Relaxed) {
                        return (Vec::new(), Vec::new());
                    }

                    let mut external_links = Vec::new();
                    let mut internal_links = Vec::new();

                    match client.get(&url_for_processing).send().await {
                        Ok(response) => {
                            if let Ok(body) = response.text().await {
                                let parser = Html::parse_document(&body);
                                let selector = Selector::parse("a[href]").unwrap();

                                for element in parser.select(&selector) {
                                    if stop_flag.load(Ordering::Relaxed) {
                                        break;
                                    }

                                    if let Some(href) = element.value().attr("href") {
                                        let full_url = Url::parse(&url_for_parser)
                                            .ok()
                                            .and_then(|u| u.join(href).ok())
                                            .map(|u| u.to_string())
                                            .unwrap_or_else(|| href.to_string());

                                        // Check if it's an external link (different domain)
                                        if let Ok(parsed) = Url::parse(&full_url) {
                                            if let Some(host) = parsed.host_str() {
                                                let host_lower = host.to_lowercase();
                                                if host_lower != base_domain {
                                                    // External link - only add if not filtered
                                                    if !filter.should_filter(&full_url) {
                                                        external_links.push(full_url);
                                                    }
                                                } else {
                                                    // Same domain, potential internal link - crawl regardless of filter
                                                    internal_links.push(full_url);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            // Skip failed requests
                        }
                    }

                    (external_links, internal_links)
                });

                handles.push(handle);
                urls_processed.push(url);
            }

            // Wait for all handles to complete
            for (url, handle) in urls_processed.into_iter().zip(handles.into_iter()) {
                if let Ok((external_links, internal_links)) = handle.await {
                    pages_crawled += 1;
                    links_found += external_links.len();

                    // Send external links through channel
                    for external_url in external_links {
                        let result = CrawlResult {
                            external_url: external_url.clone(),
                            source_url: url.clone(),
                        };
                        let stats = CrawlStats {
                            pages_crawled,
                            links_found,
                            current_url: url.clone(),
                        };
                        let _ = sender.send((result, stats)).await;
                    }

                    // Add internal links to queue (deduplicated)
                    for link in internal_links {
                        if !visited.contains(&link) {
                            queue.push_back(link);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}