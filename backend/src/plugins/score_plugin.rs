use crate::core::plugin::Plugin;
use std::sync::Arc;

#[derive(Clone)]
#[allow(dead_code)]
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

#[allow(dead_code)]
impl ScorePlugin {
    pub fn create() -> Arc<dyn Plugin> {
        Arc::new(Self)
    }
}
