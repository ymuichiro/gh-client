use crate::core::error::AppError;
use crate::core::executor::Runner;
use crate::core::observability::TraceContext;

use super::dto::TrafficOverview;
use super::service::InsightsService;

pub struct InsightsCommandHandler<R: Runner> {
    service: InsightsService<R>,
}

impl<R: Runner> InsightsCommandHandler<R> {
    pub fn new(service: InsightsService<R>) -> Self {
        Self { service }
    }

    pub fn get_views(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
    ) -> Result<TrafficOverview, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.get_views(owner, repo, &trace)
    }

    pub fn get_clones(
        &self,
        request_id: &str,
        owner: &str,
        repo: &str,
    ) -> Result<TrafficOverview, AppError> {
        let trace = TraceContext::new(request_id);
        self.service.get_clones(owner, repo, &trace)
    }
}
