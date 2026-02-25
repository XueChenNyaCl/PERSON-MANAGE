use crate::core::plugin::Plugin;
use std::sync::Arc;

#[derive(Clone)]
pub struct AttendancePlugin;

impl Plugin for AttendancePlugin {
    fn name(&self) -> &str {
        "attendance"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn initialize(&self) -> Result<(), anyhow::Error> {
        println!("Initializing attendance plugin...");
        // 这里可以实现考勤插件的初始化逻辑
        Ok(())
    }

    fn shutdown(&self) -> Result<(), anyhow::Error> {
        println!("Shutting down attendance plugin...");
        // 这里可以实现考勤插件的关闭逻辑
        Ok(())
    }
}

impl AttendancePlugin {
    pub fn new() -> Arc<dyn Plugin> {
        Arc::new(Self)
    }
}
