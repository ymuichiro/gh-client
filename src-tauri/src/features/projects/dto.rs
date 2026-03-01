use serde::{Deserialize, Serialize};

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectSummary {
    pub id: String,
    pub title: String,
    pub url: String,
    pub closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectItemSummary {
    pub id: String,
    pub content_type: Option<String>,
    pub content_title: Option<String>,
    pub content_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddedProjectItem {
    pub item_id: String,
}

#[derive(Debug, Deserialize)]
struct ProjectsResponse {
    data: ProjectsData,
}

#[derive(Debug, Deserialize)]
struct ProjectsData {
    repository: Option<ProjectsRepository>,
}

#[derive(Debug, Deserialize)]
struct ProjectsRepository {
    #[serde(rename = "projectsV2")]
    projects_v2: ProjectsConnection,
}

#[derive(Debug, Deserialize)]
struct ProjectsConnection {
    nodes: Vec<ProjectNode>,
}

#[derive(Debug, Deserialize)]
struct ProjectNode {
    id: String,
    title: String,
    url: String,
    closed: bool,
}

#[derive(Debug, Deserialize)]
struct ProjectItemsResponse {
    data: ProjectItemsData,
}

#[derive(Debug, Deserialize)]
struct ProjectItemsData {
    repository: Option<ProjectItemsRepository>,
}

#[derive(Debug, Deserialize)]
struct ProjectItemsRepository {
    #[serde(rename = "projectsV2")]
    project: Option<ProjectWithItems>,
}

#[derive(Debug, Deserialize)]
struct ProjectWithItems {
    items: ProjectItemsConnection,
}

#[derive(Debug, Deserialize)]
struct ProjectItemsConnection {
    nodes: Vec<ProjectItemNode>,
}

#[derive(Debug, Deserialize)]
struct ProjectItemNode {
    id: String,
    content: Option<ProjectItemContent>,
}

#[derive(Debug, Deserialize)]
struct ProjectItemContent {
    #[serde(rename = "__typename")]
    type_name: Option<String>,
    title: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AddItemMutationResponse {
    data: AddItemMutationData,
}

#[derive(Debug, Deserialize)]
struct AddItemMutationData {
    #[serde(rename = "addProjectV2ItemById")]
    add_project_item: AddedProjectItemNode,
}

#[derive(Debug, Deserialize)]
struct AddedProjectItemNode {
    item: AddedItem,
}

#[derive(Debug, Deserialize)]
struct AddedItem {
    id: String,
}

pub fn parse_project_summaries(payload: &str) -> Result<Vec<ProjectSummary>, AppError> {
    let parsed: ProjectsResponse = serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse projects payload: {}", err),
            false,
        )
    })?;

    Ok(parsed
        .data
        .repository
        .map(|repo| {
            repo.projects_v2
                .nodes
                .into_iter()
                .map(|node| ProjectSummary {
                    id: node.id,
                    title: node.title,
                    url: node.url,
                    closed: node.closed,
                })
                .collect()
        })
        .unwrap_or_default())
}

pub fn parse_project_item_summaries(payload: &str) -> Result<Vec<ProjectItemSummary>, AppError> {
    let parsed: ProjectItemsResponse = serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse project items payload: {}", err),
            false,
        )
    })?;

    Ok(parsed
        .data
        .repository
        .and_then(|repo| repo.project)
        .map(|project| {
            project
                .items
                .nodes
                .into_iter()
                .map(|node| ProjectItemSummary {
                    id: node.id,
                    content_type: node.content.as_ref().and_then(|c| c.type_name.clone()),
                    content_title: node.content.as_ref().and_then(|c| c.title.clone()),
                    content_url: node.content.as_ref().and_then(|c| c.url.clone()),
                })
                .collect()
        })
        .unwrap_or_default())
}

pub fn parse_added_project_item(payload: &str) -> Result<AddedProjectItem, AppError> {
    let parsed: AddItemMutationResponse = serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse project add item payload: {}", err),
            false,
        )
    })?;

    Ok(AddedProjectItem {
        item_id: parsed.data.add_project_item.item.id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_projects_payload() {
        let json = r#"{"data":{"repository":{"projectsV2":{"nodes":[{"id":"PVT_1","title":"Roadmap","url":"https://github.com/orgs/o/projects/1","closed":false}]}}}}"#;
        let items = parse_project_summaries(json).expect("payload should parse");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Roadmap");
    }

    #[test]
    fn parses_project_items_payload() {
        let json = r#"{"data":{"repository":{"projectsV2":{"items":{"nodes":[{"id":"PVTI_1","content":{"__typename":"Issue","title":"Bug","url":"https://github.com/o/r/issues/1"}}]}}}}}"#;
        let items = parse_project_item_summaries(json).expect("payload should parse");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].content_type.as_deref(), Some("Issue"));
    }

    #[test]
    fn parses_add_item_payload() {
        let json = r#"{"data":{"addProjectV2ItemById":{"item":{"id":"PVTI_1"}}}}"#;
        let item = parse_added_project_item(json).expect("payload should parse");
        assert_eq!(item.item_id, "PVTI_1");
    }
}
