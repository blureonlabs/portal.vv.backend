use async_trait::async_trait;

// TODO: Sprint — trip repository trait
#[async_trait]
pub trait Repository: Send + Sync {}
