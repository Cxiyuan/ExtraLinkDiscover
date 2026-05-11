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