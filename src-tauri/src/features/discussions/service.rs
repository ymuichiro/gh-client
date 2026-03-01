use crate::core::command_registry::CommandRegistry;
use crate::core::error::AppError;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::{PolicyGuard, RepoPermission};

use super::dto::{
    DiscussionCategory, DiscussionCreated, DiscussionSummary, parse_created_discussion,
    parse_discussion_categories, parse_discussion_summaries, parse_resolved_ids,
};

const LIST_CATEGORIES_QUERY: &str = "query($owner:String!,$repo:String!,$limit:Int!){repository(owner:$owner,name:$repo){discussionCategories(first:$limit){nodes{id name slug isAnswerable}}}}";
const LIST_DISCUSSIONS_QUERY: &str = "query($owner:String!,$repo:String!,$limit:Int!){repository(owner:$owner,name:$repo){discussions(first:$limit,orderBy:{field:UPDATED_AT,direction:DESC}){nodes{id number title url locked isAnswered category{name} author{login}}}}}";
const RESOLVE_IDS_QUERY: &str = "query($owner:String!,$repo:String!,$limit:Int!){repository(owner:$owner,name:$repo){id discussionCategories(first:$limit){nodes{id name slug isAnswerable}}}}";
const CREATE_DISCUSSION_MUTATION: &str = "mutation($repo_id:ID!,$category_id:ID!,$title:String!,$body:String!){createDiscussion(input:{repositoryId:$repo_id,categoryId:$category_id,title:$title,body:$body}){discussion{number url}}}";
const CLOSE_DISCUSSION_MUTATION: &str = "mutation($discussion_id:ID!){closeDiscussion(input:{discussionId:$discussion_id}){discussion{id}}}";
const MARK_ANSWER_MUTATION: &str = "mutation($comment_id:ID!){markDiscussionCommentAsAnswer(input:{id:$comment_id}){discussionComment{id}}}";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateDiscussionInput {
    pub owner: String,
    pub repo: String,
    pub category_slug: String,
    pub title: String,
    pub body: String,
}

impl CreateDiscussionInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.owner.trim().is_empty() || self.repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if self.category_slug.trim().is_empty() {
            return Err(AppError::validation("category_slug is required"));
        }
        if self.title.trim().is_empty() || self.body.trim().is_empty() {
            return Err(AppError::validation("title and body are required"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseDiscussionInput {
    pub discussion_id: String,
}

impl CloseDiscussionInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.discussion_id.trim().is_empty() {
            return Err(AppError::validation("discussion_id is required"));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkAnswerInput {
    pub comment_id: String,
}

impl MarkAnswerInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.comment_id.trim().is_empty() {
            return Err(AppError::validation("comment_id is required"));
        }

        Ok(())
    }
}

pub struct DiscussionsService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
    policy_guard: PolicyGuard,
}

impl<R: Runner> DiscussionsService<R> {
    pub fn new(registry: CommandRegistry, executor: CommandExecutor<R>) -> Self {
        Self {
            registry,
            executor,
            policy_guard: PolicyGuard,
        }
    }

    pub fn list_categories(
        &self,
        owner: &str,
        repo: &str,
        limit: u16,
        trace: &TraceContext,
    ) -> Result<Vec<DiscussionCategory>, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if limit == 0 {
            return Err(AppError::validation("limit must be greater than 0"));
        }

        let args = vec![
            "-f".to_string(),
            format!("query={}", LIST_CATEGORIES_QUERY),
            "-F".to_string(),
            format!("owner={}", owner),
            "-F".to_string(),
            format!("repo={}", repo),
            "-F".to_string(),
            format!("limit={}", limit),
        ];

        let req = self
            .registry
            .build_request("discussions.categories.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_discussion_categories(&output.stdout)
    }

    pub fn list(
        &self,
        owner: &str,
        repo: &str,
        limit: u16,
        trace: &TraceContext,
    ) -> Result<Vec<DiscussionSummary>, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if limit == 0 {
            return Err(AppError::validation("limit must be greater than 0"));
        }

        let args = vec![
            "-f".to_string(),
            format!("query={}", LIST_DISCUSSIONS_QUERY),
            "-F".to_string(),
            format!("owner={}", owner),
            "-F".to_string(),
            format!("repo={}", repo),
            "-F".to_string(),
            format!("limit={}", limit),
        ];

        let req = self.registry.build_request("discussions.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_discussion_summaries(&output.stdout)
    }

    pub fn create(
        &self,
        permission: RepoPermission,
        input: &CreateDiscussionInput,
        trace: &TraceContext,
    ) -> Result<DiscussionCreated, AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "discussions.create")?;
        input.validate()?;

        let resolve_args = vec![
            "-f".to_string(),
            format!("query={}", RESOLVE_IDS_QUERY),
            "-F".to_string(),
            format!("owner={}", input.owner),
            "-F".to_string(),
            format!("repo={}", input.repo),
            "-F".to_string(),
            "limit=100".to_string(),
        ];

        let resolve_req = self
            .registry
            .build_request("discussions.create", &resolve_args)?;
        let (resolve_output, _audit) = self.executor.execute(&resolve_req, trace)?;
        let resolved = parse_resolved_ids(&resolve_output.stdout, &input.category_slug)?;

        let create_args = vec![
            "-f".to_string(),
            format!("query={}", CREATE_DISCUSSION_MUTATION),
            "-F".to_string(),
            format!("repo_id={}", resolved.repository_id),
            "-F".to_string(),
            format!("category_id={}", resolved.category_id),
            "-F".to_string(),
            format!("title={}", input.title),
            "-F".to_string(),
            format!("body={}", input.body),
        ];

        let create_req = self
            .registry
            .build_request("discussions.create", &create_args)?;
        let (create_output, _audit) = self.executor.execute(&create_req, trace)?;
        parse_created_discussion(&create_output.stdout)
    }

    pub fn close(
        &self,
        permission: RepoPermission,
        input: &CloseDiscussionInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "discussions.close")?;
        input.validate()?;

        let args = vec![
            "-f".to_string(),
            format!("query={}", CLOSE_DISCUSSION_MUTATION),
            "-F".to_string(),
            format!("discussion_id={}", input.discussion_id),
        ];

        let req = self.registry.build_request("discussions.close", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }

    pub fn mark_answer(
        &self,
        permission: RepoPermission,
        input: &MarkAnswerInput,
        trace: &TraceContext,
    ) -> Result<(), AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "discussions.answer")?;
        input.validate()?;

        let args = vec![
            "-f".to_string(),
            format!("query={}", MARK_ANSWER_MUTATION),
            "-F".to_string(),
            format!("comment_id={}", input.comment_id),
        ];

        let req = self.registry.build_request("discussions.answer", &args)?;
        let _ = self.executor.execute(&req, trace)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::io;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::core::error::ErrorCode;
    use crate::core::executor::RawExecutionOutput;

    #[derive(Default)]
    struct RecordingState {
        calls: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl RecordingState {
        fn call_count(&self) -> usize {
            self.calls.lock().expect("lock poisoned").len()
        }

        fn last_call(&self) -> Option<(String, Vec<String>)> {
            self.calls.lock().expect("lock poisoned").last().cloned()
        }
    }

    struct RecordingRunner {
        state: Arc<RecordingState>,
        responses: Mutex<VecDeque<RawExecutionOutput>>,
    }

    impl RecordingRunner {
        fn new(responses: Vec<RawExecutionOutput>) -> (Self, Arc<RecordingState>) {
            let state = Arc::new(RecordingState::default());
            (
                Self {
                    state: Arc::clone(&state),
                    responses: Mutex::new(VecDeque::from(responses)),
                },
                state,
            )
        }
    }

    impl Runner for RecordingRunner {
        fn run(&self, program: &str, args: &[String]) -> io::Result<RawExecutionOutput> {
            self.state
                .calls
                .lock()
                .expect("lock poisoned")
                .push((program.to_string(), args.to_vec()));

            let response = self
                .responses
                .lock()
                .expect("lock poisoned")
                .pop_front()
                .unwrap_or(RawExecutionOutput {
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                });

            Ok(response)
        }
    }

    fn trace() -> TraceContext {
        TraceContext::new("req-discussions-service")
    }

    #[test]
    fn list_executes_query() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"data":{"repository":{"discussions":{"nodes":[{"id":"D_1","number":1,"title":"Question","url":"https://github.com/o/r/discussions/1","locked":false,"isAnswered":false,"category":{"name":"General"},"author":{"login":"octocat"}}]}}}}"#.into(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = DiscussionsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let items = service
            .list("octocat", "hello", 20, &trace())
            .expect("list should succeed");
        assert_eq!(items.len(), 1);

        let (_program, args) = state.last_call().expect("call should be recorded");
        assert!(args.contains(&"graphql".to_string()));
    }

    #[test]
    fn create_requires_write_permission() {
        let (runner, state) = RecordingRunner::new(vec![]);
        let service = DiscussionsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = CreateDiscussionInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            category_slug: "general".into(),
            title: "Question".into(),
            body: "How to use this?".into(),
        };

        let err = service
            .create(RepoPermission::Viewer, &input, &trace())
            .expect_err("viewer should be denied");

        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn create_executes_resolution_and_mutation() {
        let resolve = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"data":{"repository":{"id":"R_1","discussionCategories":{"nodes":[{"id":"DIC_1","name":"General","slug":"general","isAnswerable":true}]}}}}"#.into(),
            stderr: String::new(),
        };
        let create = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"data":{"createDiscussion":{"discussion":{"number":7,"url":"https://github.com/o/r/discussions/7"}}}}"#.into(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(vec![resolve, create]);
        let service = DiscussionsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = CreateDiscussionInput {
            owner: "octocat".into(),
            repo: "hello".into(),
            category_slug: "general".into(),
            title: "Question".into(),
            body: "How to use this?".into(),
        };

        let created = service
            .create(RepoPermission::Write, &input, &trace())
            .expect("create should succeed");
        assert_eq!(created.number, 7);
        assert_eq!(state.call_count(), 2);
    }

    #[test]
    fn mark_answer_executes_mutation() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);
        let service = DiscussionsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = MarkAnswerInput {
            comment_id: "DC_1".into(),
        };

        service
            .mark_answer(RepoPermission::Write, &input, &trace())
            .expect("mark answer should succeed");

        let (_program, args) = state.last_call().expect("call should be recorded");
        assert!(
            args.iter()
                .any(|value| value.contains("markDiscussionCommentAsAnswer"))
        );
    }
}
