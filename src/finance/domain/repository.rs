use async_trait::async_trait;

// TODO: Sprint — finance repository trait
#[async_trait]
pub trait Repository: Send + Sync {}
