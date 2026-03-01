use crate::core::command_registry::CommandRegistry;
use crate::core::error::AppError;
use crate::core::executor::{CommandExecutor, Runner};
use crate::core::observability::TraceContext;
use crate::core::policy_guard::{PolicyGuard, RepoPermission};

use super::dto::{
    AddedProjectItem, ProjectItemSummary, ProjectSummary, parse_added_project_item,
    parse_project_item_summaries, parse_project_summaries,
};

const LIST_PROJECTS_QUERY: &str = "query($owner:String!,$repo:String!,$limit:Int!){repository(owner:$owner,name:$repo){projectsV2(first:$limit,orderBy:{field:UPDATED_AT,direction:DESC}){nodes{id title url closed}}}}";
const LIST_PROJECT_ITEMS_QUERY: &str = "query($owner:String!,$repo:String!,$project_number:Int!,$limit:Int!){repository(owner:$owner,name:$repo){projectsV2(number:$project_number){items(first:$limit){nodes{id content{__typename ... on Issue {title url} ... on PullRequest {title url} ... on DraftIssue {title}}}}}}}";
const ADD_PROJECT_ITEM_MUTATION: &str = "mutation($project_id:ID!,$content_id:ID!){addProjectV2ItemById(input:{projectId:$project_id,contentId:$content_id}){item{id}}}";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddProjectItemInput {
    pub project_id: String,
    pub content_id: String,
}

impl AddProjectItemInput {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.project_id.trim().is_empty() || self.content_id.trim().is_empty() {
            return Err(AppError::validation(
                "project_id and content_id are required",
            ));
        }

        Ok(())
    }
}

pub struct ProjectsService<R: Runner> {
    registry: CommandRegistry,
    executor: CommandExecutor<R>,
    policy_guard: PolicyGuard,
}

impl<R: Runner> ProjectsService<R> {
    pub fn new(registry: CommandRegistry, executor: CommandExecutor<R>) -> Self {
        Self {
            registry,
            executor,
            policy_guard: PolicyGuard,
        }
    }

    pub fn list(
        &self,
        owner: &str,
        repo: &str,
        limit: u16,
        trace: &TraceContext,
    ) -> Result<Vec<ProjectSummary>, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if limit == 0 {
            return Err(AppError::validation("limit must be greater than 0"));
        }

        let args = vec![
            "-f".to_string(),
            format!("query={}", LIST_PROJECTS_QUERY),
            "-F".to_string(),
            format!("owner={}", owner),
            "-F".to_string(),
            format!("repo={}", repo),
            "-F".to_string(),
            format!("limit={}", limit),
        ];

        let req = self.registry.build_request("projects.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_project_summaries(&output.stdout)
    }

    pub fn list_items(
        &self,
        owner: &str,
        repo: &str,
        project_number: u32,
        limit: u16,
        trace: &TraceContext,
    ) -> Result<Vec<ProjectItemSummary>, AppError> {
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(AppError::validation("owner and repo are required"));
        }
        if project_number == 0 {
            return Err(AppError::validation(
                "project_number must be greater than 0",
            ));
        }
        if limit == 0 {
            return Err(AppError::validation("limit must be greater than 0"));
        }

        let args = vec![
            "-f".to_string(),
            format!("query={}", LIST_PROJECT_ITEMS_QUERY),
            "-F".to_string(),
            format!("owner={}", owner),
            "-F".to_string(),
            format!("repo={}", repo),
            "-F".to_string(),
            format!("project_number={}", project_number),
            "-F".to_string(),
            format!("limit={}", limit),
        ];

        let req = self.registry.build_request("projects.items.list", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_project_item_summaries(&output.stdout)
    }

    pub fn add_item(
        &self,
        permission: RepoPermission,
        input: &AddProjectItemInput,
        trace: &TraceContext,
    ) -> Result<AddedProjectItem, AppError> {
        self.policy_guard
            .require(RepoPermission::Write, permission, "projects.items.add")?;
        input.validate()?;

        let args = vec![
            "-f".to_string(),
            format!("query={}", ADD_PROJECT_ITEM_MUTATION),
            "-F".to_string(),
            format!("project_id={}", input.project_id),
            "-F".to_string(),
            format!("content_id={}", input.content_id),
        ];

        let req = self.registry.build_request("projects.items.add", &args)?;
        let (output, _audit) = self.executor.execute(&req, trace)?;
        parse_added_project_item(&output.stdout)
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
        TraceContext::new("req-projects-service")
    }

    #[test]
    fn list_executes_projects_query() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"data":{"repository":{"projectsV2":{"nodes":[{"id":"PVT_1","title":"Roadmap","url":"https://github.com/orgs/o/projects/1","closed":false}]}}}}"#.into(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = ProjectsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let items = service
            .list("octocat", "hello", 10, &trace())
            .expect("list should succeed");
        assert_eq!(items.len(), 1);

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(args.contains(&"graphql".to_string()));
        assert!(args.iter().any(|value| value.contains("projectsV2")));
    }

    #[test]
    fn add_item_requires_write_permission() {
        let (runner, state) = RecordingRunner::new(vec![RawExecutionOutput {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }]);

        let service = ProjectsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = AddProjectItemInput {
            project_id: "PVT_1".into(),
            content_id: "PR_1".into(),
        };

        let err = service
            .add_item(RepoPermission::Viewer, &input, &trace())
            .expect_err("viewer should be denied");

        assert_eq!(err.code, ErrorCode::PermissionDenied);
        assert_eq!(state.call_count(), 0);
    }

    #[test]
    fn add_item_executes_mutation() {
        let output = RawExecutionOutput {
            exit_code: 0,
            stdout: r#"{"data":{"addProjectV2ItemById":{"item":{"id":"PVTI_1"}}}}"#.into(),
            stderr: String::new(),
        };

        let (runner, state) = RecordingRunner::new(vec![output]);
        let service = ProjectsService::new(
            CommandRegistry::with_defaults(),
            CommandExecutor::new(runner, false),
        );

        let input = AddProjectItemInput {
            project_id: "PVT_1".into(),
            content_id: "PR_1".into(),
        };

        let created = service
            .add_item(RepoPermission::Write, &input, &trace())
            .expect("add should succeed");
        assert_eq!(created.item_id, "PVTI_1");

        let (_program, args) = state.last_call().expect("command should be called");
        assert!(
            args.iter()
                .any(|value| value.contains("addProjectV2ItemById"))
        );
    }
}
