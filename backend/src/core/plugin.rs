use dyn_clone::DynClone;
use std::sync::Arc;

pub trait Plugin: DynClone + Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&self) -> Result<(), anyhow::Error>;
    fn shutdown(&self) -> Result<(), anyhow::Error>;
}

dyn_clone::clone_trait_object!(Plugin);

#[derive(Clone)]
pub struct PluginManager {
    plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn initialize_all(&self) -> Result<(), anyhow::Error> {
        for plugin in &self.plugins {
            plugin.initialize()?;
        }
        Ok(())
    }

    pub fn shutdown_all(&self) -> Result<(), anyhow::Error> {
        for plugin in &self.plugins {
            plugin.shutdown()?;
        }
        Ok(())
    }

    pub fn get_plugin(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.iter().find(|p| p.name() == name).cloned()
    }
}
