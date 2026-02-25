use crate::core::plugin::Plugin;
use std::sync::Arc;

#[derive(Clone)]
pub struct NoticePlugin;

impl Plugin for NoticePlugin {
    fn name(&self) -> &str {
        "notice"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn initialize(&self) -> Result<(), anyhow::Error> {
        println!("Initializing notice plugin...");
        // 这里可以实现通知公告插件的初始化逻辑
        Ok(())
    }

    fn shutdown(&self) -> Result<(), anyhow::Error> {
        println!("Shutting down notice plugin...");
        // 这里可以实现通知公告插件的关闭逻辑
        Ok(())
    }
}

impl NoticePlugin {
    pub fn new() -> Arc<dyn Plugin> {
        Arc::new(Self)
    }
}
