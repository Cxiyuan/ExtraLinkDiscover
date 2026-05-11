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
                    // Handle wildcard patterns like *.nczy.edu.cn
                    if domain.starts_with("*.") {
                        let base_domain = &domain[2..]; // Remove "*."
                        // Match if host equals base_domain or ends with ".base_domain"
                        if host_lower == base_domain || host_lower.ends_with(&format!(".{}", base_domain)) {
                            return true;
                        }
                    } else {
                        // Normal domain matching
                        if host_lower == *domain || host_lower.ends_with(&format!(".{}", domain)) {
                            return true;
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