use async_trait::async_trait;

// TODO: Sprint — uber repository trait
#[allow(dead_code)]
#[async_trait]
pub trait Repository: Send + Sync {}
