/// Tab operations: sorting, uniqueness, etc.

use crate::domain::extract_domain;
use crate::tab_data::TabInfo;

/// Sort tabs by domain (precompute domain for each tab)
pub fn sort_tabs_by_domain(tabs: &[TabInfo]) -> Vec<TabInfo> {
    let mut tabs_with_domain: Vec<(TabInfo, String)> = tabs
        .iter()
        .filter_map(|tab: &TabInfo| {
            match extract_domain(&tab.url) {
                Some(domain) => Some((tab.clone(), domain)),
                None => None,
            }
        })
        .collect();

    // Sort by domain, then by URL
    tabs_with_domain.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.url.cmp(&b.0.url)));

    tabs_with_domain.into_iter().map(|(tab, _)| tab).collect()
}

/// Make tabs unique by URL (keep first occurrence)
pub fn make_tabs_unique(tabs: &[TabInfo]) -> (Vec<TabInfo>, Vec<i32>) {
    let mut seen_urls = std::collections::HashSet::new();
    let mut keep_tabs = Vec::new();
    let mut remove_ids = Vec::new();

    for tab in tabs {
        if seen_urls.contains(&tab.url) {
            remove_ids.push(tab.id);
        } else {
            seen_urls.insert(tab.url.clone());
            keep_tabs.push(tab.clone());
        }
    }

    (keep_tabs, remove_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tab(id: i32, url: &str, title: &str) -> TabInfo {
        TabInfo {
            id,
            url: url.to_string(),
            title: title.to_string(),
            pinned: false,
            index: id,
        }
    }

    #[test]
    fn test_sort_tabs_by_domain() {
        let tabs = vec![
            create_test_tab(1, "https://github.com/rust", "GitHub Rust"),
            create_test_tab(2, "https://www.google.com", "Google"),
            create_test_tab(3, "https://docs.microsoft.com", "Microsoft Docs"),
            create_test_tab(4, "https://mail.google.com", "Gmail"),
        ];

        let sorted = sort_tabs_by_domain(&tabs);

        // Should be sorted: github, google, google, microsoft
        assert_eq!(extract_domain(&sorted[0].url), Some("github.com".to_string()));
        assert_eq!(extract_domain(&sorted[1].url), Some("google.com".to_string()));
        assert_eq!(extract_domain(&sorted[2].url), Some("google.com".to_string()));
        assert_eq!(extract_domain(&sorted[3].url), Some("microsoft.com".to_string()));
    }

    #[test]
    fn test_make_tabs_unique() {
        let tabs = vec![
            create_test_tab(1, "https://google.com", "Google 1"),
            create_test_tab(2, "https://github.com", "GitHub"),
            create_test_tab(3, "https://google.com", "Google 2"), // duplicate
            create_test_tab(4, "https://microsoft.com", "Microsoft"),
            create_test_tab(5, "https://github.com", "GitHub 2"), // duplicate
        ];

        let (keep, remove) = make_tabs_unique(&tabs);

        assert_eq!(keep.len(), 3); // google, github, microsoft
        assert_eq!(remove.len(), 2); // IDs 3 and 5
        assert!(remove.contains(&3));
        assert!(remove.contains(&5));
    }

    #[test]
    fn test_make_tabs_unique_no_duplicates() {
        let tabs = vec![
            create_test_tab(1, "https://google.com", "Google"),
            create_test_tab(2, "https://github.com", "GitHub"),
            create_test_tab(3, "https://microsoft.com", "Microsoft"),
        ];

        let (keep, remove) = make_tabs_unique(&tabs);

        assert_eq!(keep.len(), 3);
        assert_eq!(remove.len(), 0);
    }
}
