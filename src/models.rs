use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub link: Option<String>,   // URL to the artifact (Google Doc, Video, etc.)
    pub link_text: String,      // "View Document", "Watch Video"
    pub icon: String,           // SVG path or icon name
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompetencyBadge {
    pub title: String,
    pub description: String,
    pub artifacts: Vec<Artifact>,
}