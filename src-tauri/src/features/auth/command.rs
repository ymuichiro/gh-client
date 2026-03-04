use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;

use super::dto::{GhAuthStatus, GhOrganization};
use super::service::AuthService;

pub struct AuthCommandHandler<R: Runner> {
    service: AuthService<R>,
}

impl<R: Runner> AuthCommandHandler<R> {
    pub fn new(service: AuthService<R>) -> Self {
        Self { service }
    }

    pub fn status(&self, request_id: &str) -> Result<GhAuthStatus, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.status(&trace)
    }

    pub fn organizations_list(&self, request_id: &str) -> Result<Vec<GhOrganization>, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.list_organizations(&trace)
    }
}
