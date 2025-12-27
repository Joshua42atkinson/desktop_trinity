use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};

/// Web Browser Skill
///
/// Provides capabilities to fetch web pages and extract content.
pub struct WebBrowser {
    client: Client,
}

impl WebBrowser {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Trinity/1.0 (AI Agent)")
                .build()
                .unwrap_or_default(),
        }
    }

    /// Fetch a URL and return the text content (cleaned)
    pub async fn browse(&self, url: &str) -> Result<String> {
        let resp = self.client.get(url).send().await?;
        let status = resp.status();
        
        if !status.is_success() {
            anyhow::bail!("Failed to fetch URL {}: {}", url, status);
        }

        let html_content = resp.text().await?;
        let document = Html::parse_document(&html_content);
        
        // Extract meaningful text
        // TODO: Improve extraction logic (remove scripts, styles, etc.)
        let selector = Selector::parse("body").unwrap();
        
        let text = if let Some(body) = document.select(&selector).next() {
            body.text().collect::<Vec<_>>().join(" ")
        } else {
            "No content found".to_string()
        };

        // Clean up whitespace
        let cleaned: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
        
        Ok(cleaned)
    }

    /// Perform a search (placeholder - requires Search API key usually)
    pub async fn search(&self, query: &str) -> Result<Vec<String>> {
        // For now, we just return a stub. 
        // Real implementation would use Google/Bing/SerpApi
        Ok(vec![format!("Results for: {}", query)])
    }
}
