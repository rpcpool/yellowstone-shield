pub mod policy;
pub mod validator;

use crate::CommandContext;
use anyhow::Result;

#[async_trait::async_trait]
pub trait RunCommand {
    async fn run(&self, context: CommandContext) -> Result<()>;
}
