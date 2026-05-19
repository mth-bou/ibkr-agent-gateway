//! Broker news read models.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// News metadata row.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NewsArticleMeta {
    /// Article id.
    pub article_id: String,
    /// Headline.
    pub headline: String,
    /// Provider code.
    pub provider: Option<String>,
    /// Published timestamp as broker-provided RFC3339 text.
    pub published_at: Option<String>,
}

/// News list response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NewsList {
    /// Symbol queried.
    pub symbol: String,
    /// Article metadata.
    pub articles: Vec<NewsArticleMeta>,
}

/// News article response.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct NewsArticle {
    /// Article id.
    pub article_id: String,
    /// Headline.
    pub headline: String,
    /// Body text.
    pub body: String,
    /// Provider code.
    pub provider: Option<String>,
}
