use async_trait::async_trait;

// TODO: Sprint — hr repository trait
#[async_trait]
pub trait Repository: Send + Sync {}
