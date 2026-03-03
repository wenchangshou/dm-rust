use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::device::DeviceController;

pub type SharedController = Arc<RwLock<DeviceController>>;
pub type SharedConfig = Arc<RwLock<Config>>;
pub type SharedConfigPath = Arc<String>;
