use tracing::{info, warn, error, debug};

/// 日志辅助函数
pub struct Logger;

impl Logger {
    pub fn info(msg: &str) {
        info!("{}", msg);
    }

    pub fn warn(msg: &str) {
        warn!("{}", msg);
    }

    pub fn error(msg: &str) {
        error!("{}", msg);
    }

    pub fn debug(msg: &str) {
        debug!("{}", msg);
    }
}
