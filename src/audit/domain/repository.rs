use async_trait::async_trait;

// TODO: Sprint — audit repository trait
#[async_trait]
pub trait Repository: Send + Sync {}
