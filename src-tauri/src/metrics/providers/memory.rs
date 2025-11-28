//! Memory metrics provider using sysinfo

use crate::core::error::MetricsError;
use crate::core::interfaces::MemoryMetricsProvider;
use crate::metrics::models::MemoryMetrics;
use async_trait::async_trait;
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::Mutex;

/// Memory metrics provider using sysinfo
pub struct SysInfoMemoryMetricsProvider {
    system: Arc<Mutex<System>>,
}

impl SysInfoMemoryMetricsProvider {
    pub fn new(system: Arc<Mutex<System>>) -> Self {
        Self { system }
    }
}

#[async_trait]
impl MemoryMetricsProvider for SysInfoMemoryMetricsProvider {
    async fn get_memory_metrics(&self) -> Result<MemoryMetrics, MetricsError> {
        let mut system = self.system.lock().await;
        system.refresh_memory();
        
        let total_mb = system.total_memory() / (1024 * 1024); // bytes to MB
        let used_mb = system.used_memory() / (1024 * 1024); // bytes to MB
        
        let swap_total_mb = system.total_swap() / (1024 * 1024);
        let swap_used_mb = system.used_swap() / (1024 * 1024);
        
        Ok(MemoryMetrics {
            used_mb,
            total_mb,
            swap_used_mb: if swap_total_mb > 0 { Some(swap_used_mb) } else { None },
            swap_total_mb: if swap_total_mb > 0 { Some(swap_total_mb) } else { None },
        })
    }
}

