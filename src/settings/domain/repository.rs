use async_trait::async_trait;

// TODO: Sprint — settings repository trait
#[async_trait]
pub trait Repository: Send + Sync {}
