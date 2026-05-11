use url::Url;

pub struct DomainFilter {
    blocked_domains: Vec<String>,
}

impl DomainFilter {
    pub fn new(input: &str) -> Self {
        // Support both comma and newline separated domains
        let domains: Vec<String> = input
            .replace('，', ",")
            .split(|c| c == ',' || c == '\n')
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
                    if domain.starts_with("*.") {
                        let base_domain = &domain[2..];
                        if host_lower == base_domain || host_lower.ends_with(&format!(".{}", base_domain)) {
                            return true;
                        }
                    } else if !domain.is_empty() {
                        // Non-wildcard: only match exact domain or immediate subdomains
                        // Use ends_with but require the prefix before . is the same as domain
                        if host_lower == *domain {
                            return true;
                        }
                        // Only match subdomains if the prefix matches the domain exactly
                        // e.g., "example.com" matches "www.example.com" but NOT "notexample.com"
                        if host_lower.ends_with(&format!(".{}", domain)) {
                            let prefix = &host_lower[..host_lower.len() - domain.len() - 1];
                            if !prefix.is_empty() && !prefix.contains('.') {
                                return true;
                            }
                        }
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