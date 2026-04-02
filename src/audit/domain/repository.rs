use async_trait::async_trait;

// TODO: Sprint — audit repository trait
#[allow(dead_code)]
#[async_trait]
pub trait Repository: Send + Sync {}
