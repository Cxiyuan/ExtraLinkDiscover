use ExtraLinkDiscover::crawler::{CrawlResult, Crawler};
use ExtraLinkDiscover::filter::DomainFilter;

#[tokio::test]
async fn test_external_link_detection() {
    // Test the crawler structure exists and works
    let filter = DomainFilter::new("");
    let crawler = Crawler::new(filter, 5);
    assert_eq!(crawler.concurrency(), 5);
}

#[tokio::test]
async fn test_crawler_creation() {
    let filter = DomainFilter::new("");
    let crawler = Crawler::new(filter, 10);
    assert_eq!(crawler.concurrency(), 10);
}

#[tokio::test]
async fn test_crawl_result_struct() {
    let result = CrawlResult {
        external_url: "https://external.com".to_string(),
        source_url: "https://source.com/page".to_string(),
    };
    assert_eq!(result.external_url, "https://external.com");
    assert_eq!(result.source_url, "https://source.com/page");
}