/// Domain extraction and counting logic for Tab Hoarder
use std::collections::HashMap;

/// Extract the domain from a URL with smart TLD handling
///
/// Algorithm:
/// 1. Parse URL to extract hostname
/// 2. Split hostname by "."
/// 3. Get last segment (TLD)
/// 4. If TLD is 2 letters AND second-to-last is "co" or "com":
///    → Return last 3 segments (e.g., "example.com.au", "site.co.uk")
/// 5. Else:
///    → Return last 2 segments (e.g., "microsoft.com", "zinfandel.io")
/// 6. Handle edge cases (localhost, IPs, etc.)
///
/// Examples:
/// - https://www.google.com/search → google.com
/// - https://ai.microsoft.com → microsoft.com
/// - https://news.bbc.co.uk/article → bbc.co.uk
/// - https://shop.example.com.au/products → example.com.au
pub fn extract_domain(url: &str) -> String {
    // Handle empty or invalid URLs
    if url.is_empty() {
        return String::from("invalid");
    }

    // Extract hostname from URL
    let hostname = match extract_hostname(url) {
        Some(h) => h,
        None => return String::from("invalid"),
    };

    // Handle localhost and IP addresses
    if hostname == "localhost" || is_ip_address(&hostname) {
        return hostname;
    }

    // Split hostname into parts
    let parts: Vec<&str> = hostname.split('.').collect();

    // Need at least 2 parts for a valid domain
    if parts.len() < 2 {
        return hostname;
    }

    // Get the TLD (last part)
    let tld = parts[parts.len() - 1];

    // Check if we need 3 parts (for .co.uk, .com.au style TLDs)
    if parts.len() >= 3 && tld.len() == 2 {
        let second_level = parts[parts.len() - 2];
        if second_level == "co" || second_level == "com" {
            // Return last 3 parts (e.g., "bbc.co.uk" or "example.com.au")
            return parts[parts.len() - 3..].join(".");
        }
    }

    // Default: return last 2 parts (e.g., "google.com")
    parts[parts.len() - 2..].join(".")
}

/// Extract hostname from a URL string
fn extract_hostname(url: &str) -> Option<String> {
    // Remove protocol if present
    let url_clean = url
        .trim()
        .replace("https://", "")
        .replace("http://", "")
        .replace("ftp://", "");

    // Get everything before the first '/' (or the whole string if no '/')
    let hostname_with_port = url_clean.split('/').next()?.to_string();

    // Remove port if present (e.g., "localhost:3000" -> "localhost")
    let hostname = hostname_with_port
        .split(':')
        .next()?
        .to_lowercase();

    if hostname.is_empty() {
        None
    } else {
        Some(hostname)
    }
}

/// Check if a string looks like an IP address
fn is_ip_address(s: &str) -> bool {
    // Simple check: if it starts with a digit and contains only digits and dots
    s.chars().next().map_or(false, |c| c.is_ascii_digit())
        && s.chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// Count domain occurrences from a list of URLs
pub fn count_domains(urls: &[String]) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for url in urls {
        let domain = extract_domain(url);
        *counts.entry(domain).or_insert(0) += 1;
    }

    counts
}

/// Get the top N domains by count
pub fn get_top_domains(counts: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    let mut domain_vec: Vec<(String, usize)> = counts
        .iter()
        .map(|(domain, count)| (domain.clone(), *count))
        .collect();

    // Sort by count descending, then by domain name ascending
    domain_vec.sort_by(|a, b| {
        b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))
    });

    // Take top N
    domain_vec.into_iter().take(n).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain_basic() {
        assert_eq!(extract_domain("https://www.google.com"), "google.com");
        assert_eq!(extract_domain("https://google.com"), "google.com");
        assert_eq!(extract_domain("http://google.com"), "google.com");
    }

    #[test]
    fn test_extract_domain_subdomains() {
        assert_eq!(extract_domain("https://ai.microsoft.com"), "microsoft.com");
        assert_eq!(extract_domain("https://app.microsoft.com"), "microsoft.com");
        assert_eq!(extract_domain("https://docs.microsoft.com"), "microsoft.com");
        assert_eq!(extract_domain("https://www.microsoft.com"), "microsoft.com");
    }

    #[test]
    fn test_extract_domain_with_path() {
        assert_eq!(extract_domain("https://www.google.com/search?q=rust"), "google.com");
        assert_eq!(extract_domain("https://github.com/rust-lang/rust"), "github.com");
    }

    #[test]
    fn test_extract_domain_country_tlds() {
        assert_eq!(extract_domain("https://news.bbc.co.uk"), "bbc.co.uk");
        assert_eq!(extract_domain("https://www.bbc.co.uk/news"), "bbc.co.uk");
        assert_eq!(extract_domain("https://shop.example.com.au"), "example.com.au");
        assert_eq!(extract_domain("https://store.amazon.com.au"), "amazon.com.au");
    }

    #[test]
    fn test_extract_domain_special_cases() {
        assert_eq!(extract_domain("https://localhost:3000"), "localhost");
        assert_eq!(extract_domain("http://127.0.0.1:8080"), "127.0.0.1");
        assert_eq!(extract_domain("https://192.168.1.1"), "192.168.1.1");
    }

    #[test]
    fn test_extract_domain_edge_cases() {
        assert_eq!(extract_domain(""), "invalid");
        assert_eq!(extract_domain("not-a-url"), "not-a-url");
        assert_eq!(extract_domain("https://"), "invalid");
    }

    #[test]
    fn test_extract_domain_io_domains() {
        assert_eq!(extract_domain("https://zinfandel.io"), "zinfandel.io");
        assert_eq!(extract_domain("https://api.zinfandel.io"), "zinfandel.io");
    }

    #[test]
    fn test_count_domains() {
        let urls = vec![
            "https://www.google.com/search".to_string(),
            "https://mail.google.com".to_string(),
            "https://github.com/rust".to_string(),
            "https://www.google.com/maps".to_string(),
            "https://github.com/yewstack".to_string(),
        ];

        let counts = count_domains(&urls);

        assert_eq!(counts.get("google.com"), Some(&3));
        assert_eq!(counts.get("github.com"), Some(&2));
    }

    #[test]
    fn test_get_top_domains() {
        let mut counts = HashMap::new();
        counts.insert("google.com".to_string(), 10);
        counts.insert("github.com".to_string(), 5);
        counts.insert("microsoft.com".to_string(), 8);
        counts.insert("reddit.com".to_string(), 3);

        let top3 = get_top_domains(&counts, 3);

        assert_eq!(top3.len(), 3);
        assert_eq!(top3[0], ("google.com".to_string(), 10));
        assert_eq!(top3[1], ("microsoft.com".to_string(), 8));
        assert_eq!(top3[2], ("github.com".to_string(), 5));
    }

    #[test]
    fn test_get_top_domains_with_ties() {
        let mut counts = HashMap::new();
        counts.insert("github.com".to_string(), 5);
        counts.insert("google.com".to_string(), 5);
        counts.insert("microsoft.com".to_string(), 5);

        let top2 = get_top_domains(&counts, 2);

        // With same counts, should be sorted alphabetically
        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].0, "github.com");
        assert_eq!(top2[1].0, "google.com");
    }
}
