use crate::core::plugin::Plugin;
use std::sync::Arc;

#[derive(Clone)]
pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn name(&self) -> &str {
        "score"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn initialize(&self) -> Result<(), anyhow::Error> {
        println!("Initializing score plugin...");
        // 这里可以实现评分插件的初始化逻辑
        Ok(())
    }

    fn shutdown(&self) -> Result<(), anyhow::Error> {
        println!("Shutting down score plugin...");
        // 这里可以实现评分插件的关闭逻辑
        Ok(())
    }
}

impl ScorePlugin {
    pub fn new() -> Arc<dyn Plugin> {
        Arc::new(Self)
    }
}
