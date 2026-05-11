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