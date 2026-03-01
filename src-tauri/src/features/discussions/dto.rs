use serde::{Deserialize, Serialize};

use crate::core::error::{AppError, ErrorCode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscussionCategory {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub is_answerable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscussionSummary {
    pub id: String,
    pub number: u64,
    pub title: String,
    pub url: String,
    pub locked: Option<bool>,
    pub is_answered: Option<bool>,
    pub category: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscussionCreated {
    pub number: u64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscussionResolvedIds {
    pub repository_id: String,
    pub category_id: String,
}

#[derive(Debug, Deserialize)]
struct CategoriesResponse {
    data: CategoriesData,
}

#[derive(Debug, Deserialize)]
struct CategoriesData {
    repository: Option<CategoriesRepository>,
}

#[derive(Debug, Deserialize)]
struct CategoriesRepository {
    #[serde(rename = "discussionCategories")]
    categories: DiscussionCategoryConnection,
}

#[derive(Debug, Deserialize)]
struct DiscussionCategoryConnection {
    nodes: Vec<DiscussionCategoryNode>,
}

#[derive(Debug, Deserialize)]
struct DiscussionCategoryNode {
    id: String,
    name: String,
    slug: String,
    #[serde(rename = "isAnswerable")]
    is_answerable: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct DiscussionsResponse {
    data: DiscussionsData,
}

#[derive(Debug, Deserialize)]
struct DiscussionsData {
    repository: Option<DiscussionsRepository>,
}

#[derive(Debug, Deserialize)]
struct DiscussionsRepository {
    discussions: DiscussionConnection,
}

#[derive(Debug, Deserialize)]
struct DiscussionConnection {
    nodes: Vec<DiscussionNode>,
}

#[derive(Debug, Deserialize)]
struct DiscussionNode {
    id: String,
    number: u64,
    title: String,
    url: String,
    locked: Option<bool>,
    #[serde(rename = "isAnswered")]
    is_answered: Option<bool>,
    category: Option<DiscussionCategoryRef>,
    author: Option<DiscussionAuthor>,
}

#[derive(Debug, Deserialize)]
struct DiscussionCategoryRef {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DiscussionAuthor {
    login: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResolveRepositoryCategoryResponse {
    data: ResolveRepositoryCategoryData,
}

#[derive(Debug, Deserialize)]
struct ResolveRepositoryCategoryData {
    repository: Option<RepositoryWithCategories>,
}

#[derive(Debug, Deserialize)]
struct RepositoryWithCategories {
    id: String,
    #[serde(rename = "discussionCategories")]
    categories: DiscussionCategoryConnection,
}

#[derive(Debug, Deserialize)]
struct CreateDiscussionMutationResponse {
    data: CreateDiscussionMutationData,
}

#[derive(Debug, Deserialize)]
struct CreateDiscussionMutationData {
    #[serde(rename = "createDiscussion")]
    create_discussion: CreateDiscussionPayload,
}

#[derive(Debug, Deserialize)]
struct CreateDiscussionPayload {
    discussion: CreatedDiscussionNode,
}

#[derive(Debug, Deserialize)]
struct CreatedDiscussionNode {
    number: u64,
    url: String,
}

pub fn parse_discussion_categories(payload: &str) -> Result<Vec<DiscussionCategory>, AppError> {
    let parsed: CategoriesResponse = serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse discussion categories payload: {}", err),
            false,
        )
    })?;

    Ok(parsed
        .data
        .repository
        .map(|repo| {
            repo.categories
                .nodes
                .into_iter()
                .map(|node| DiscussionCategory {
                    id: node.id,
                    name: node.name,
                    slug: node.slug,
                    is_answerable: node.is_answerable,
                })
                .collect()
        })
        .unwrap_or_default())
}

pub fn parse_discussion_summaries(payload: &str) -> Result<Vec<DiscussionSummary>, AppError> {
    let parsed: DiscussionsResponse = serde_json::from_str(payload).map_err(|err| {
        AppError::new(
            ErrorCode::UpstreamError,
            format!("failed to parse discussions payload: {}", err),
            false,
        )
    })?;

    Ok(parsed
        .data
        .repository
        .map(|repo| {
            repo.discussions
                .nodes
                .into_iter()
                .map(|node| DiscussionSummary {
                    id: node.id,
                    number: node.number,
                    title: node.title,
                    url: node.url,
                    locked: node.locked,
                    is_answered: node.is_answered,
                    category: node.category.and_then(|value| value.name),
                    author: node.author.and_then(|value| value.login),
                })
                .collect()
        })
        .unwrap_or_default())
}

pub fn parse_resolved_ids(
    payload: &str,
    category_slug: &str,
) -> Result<DiscussionResolvedIds, AppError> {
    let parsed: ResolveRepositoryCategoryResponse =
        serde_json::from_str(payload).map_err(|err| {
            AppError::new(
                ErrorCode::UpstreamError,
                format!("failed to parse discussion id resolution payload: {}", err),
                false,
            )
        })?;

    let repository = parsed.data.repository.ok_or_else(|| {
        AppError::new(
            ErrorCode::NotFound,
            "repository not found while resolving discussion ids",
            false,
        )
    })?;

    let category = repository
        .categories
        .nodes
        .into_iter()
        .find(|node| node.slug == category_slug)
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::NotFound,
                format!("discussion category `{}` not found", category_slug),
                false,
            )
        })?;

    Ok(DiscussionResolvedIds {
        repository_id: repository.id,
        category_id: category.id,
    })
}

pub fn parse_created_discussion(payload: &str) -> Result<DiscussionCreated, AppError> {
    let parsed: CreateDiscussionMutationResponse =
        serde_json::from_str(payload).map_err(|err| {
            AppError::new(
                ErrorCode::UpstreamError,
                format!("failed to parse create discussion payload: {}", err),
                false,
            )
        })?;

    Ok(DiscussionCreated {
        number: parsed.data.create_discussion.discussion.number,
        url: parsed.data.create_discussion.discussion.url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_categories_payload() {
        let json = r#"{"data":{"repository":{"discussionCategories":{"nodes":[{"id":"DIC_1","name":"General","slug":"general","isAnswerable":true}]}}}}"#;
        let categories = parse_discussion_categories(json).expect("payload should parse");
        assert_eq!(categories.len(), 1);
        assert_eq!(categories[0].slug, "general");
    }

    #[test]
    fn parses_discussions_payload() {
        let json = r#"{"data":{"repository":{"discussions":{"nodes":[{"id":"D_1","number":1,"title":"Question","url":"https://github.com/o/r/discussions/1","locked":false,"isAnswered":false,"category":{"name":"General"},"author":{"login":"octocat"}}]}}}}"#;
        let discussions = parse_discussion_summaries(json).expect("payload should parse");
        assert_eq!(discussions.len(), 1);
        assert_eq!(discussions[0].number, 1);
    }

    #[test]
    fn parses_resolved_ids_payload() {
        let json = r#"{"data":{"repository":{"id":"R_1","discussionCategories":{"nodes":[{"id":"DIC_1","name":"General","slug":"general","isAnswerable":true}]}}}}"#;
        let ids = parse_resolved_ids(json, "general").expect("payload should parse");
        assert_eq!(ids.repository_id, "R_1");
        assert_eq!(ids.category_id, "DIC_1");
    }

    #[test]
    fn parses_created_discussion_payload() {
        let json = r#"{"data":{"createDiscussion":{"discussion":{"number":5,"url":"https://github.com/o/r/discussions/5"}}}}"#;
        let created = parse_created_discussion(json).expect("payload should parse");
        assert_eq!(created.number, 5);
    }
}
