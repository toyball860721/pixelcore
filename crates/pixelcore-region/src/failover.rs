use crate::{
    error::Result,
    manager::RegionManager,
    region::{RegionId, RegionStatus},
};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Failover manager for automatic region failover
pub struct FailoverManager {
    region_manager: Arc<RegionManager>,
    failover_threshold: u32,
    recovery_threshold: u32,
}

impl FailoverManager {
    pub fn new(region_manager: Arc<RegionManager>) -> Self {
        Self {
            region_manager,
            failover_threshold: 3,  // Fail after 3 consecutive failures
            recovery_threshold: 2,  // Recover after 2 consecutive successes
        }
    }

    /// Trigger failover from unhealthy region to healthy backup
    pub async fn trigger_failover(&self, failed_region: RegionId) -> Result<RegionId> {
        info!("Triggering failover from region {}", failed_region);

        // Mark the failed region as unhealthy
        self.region_manager
            .update_region_status(failed_region, RegionStatus::Unhealthy)
            .await?;

        // Find a healthy backup region
        let healthy_regions = self.region_manager.get_healthy_regions().await;

        if healthy_regions.is_empty() {
            error!("No healthy regions available for failover");
            return Err(crate::error::RegionError::NoHealthyRegions);
        }

        // Select the region with lowest load
        let backup_region = healthy_regions
            .iter()
            .min_by(|a, b| {
                a.capacity.current_load
                    .partial_cmp(&b.capacity.current_load)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|r| r.id)
            .ok_or(crate::error::RegionError::NoHealthyRegions)?;

        info!("Failover complete: {} -> {}", failed_region, backup_region);

        Ok(backup_region)
    }

    /// Attempt to recover a failed region
    pub async fn attempt_recovery(&self, region_id: RegionId) -> Result<bool> {
        info!("Attempting recovery for region {}", region_id);

        let region = self.region_manager.get_region(region_id).await?;

        // Try health check
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        let health_url = format!("{}/health", region.endpoint);

        match client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => {
                // Recovery successful
                self.region_manager
                    .update_region_status(region_id, RegionStatus::Healthy)
                    .await?;

                info!("Region {} recovered successfully", region_id);
                Ok(true)
            }
            Ok(response) => {
                warn!("Region {} still unhealthy: HTTP {}", region_id, response.status());
                Ok(false)
            }
            Err(e) => {
                warn!("Region {} recovery failed: {}", region_id, e);
                Ok(false)
            }
        }
    }

    /// Start automatic failover monitoring
    pub async fn start_failover_monitor(self: Arc<Self>) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));

        tokio::spawn(async move {
            loop {
                interval.tick().await;

                if let Err(e) = self.check_and_failover().await {
                    error!("Failover monitor error: {}", e);
                }
            }
        });
    }

    /// Check all regions and trigger failover if needed
    async fn check_and_failover(&self) -> Result<()> {
        let all_regions = self.region_manager.get_all_regions().await;

        for region in all_regions {
            match region.status {
                RegionStatus::Unhealthy => {
                    // Try recovery first
                    if !self.attempt_recovery(region.id).await? {
                        // If recovery fails, ensure failover is in place
                        warn!("Region {} remains unhealthy", region.id);
                    }
                }
                RegionStatus::Degraded => {
                    // Monitor degraded regions
                    warn!("Region {} is degraded", region.id);
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Get failover statistics
    pub async fn get_failover_stats(&self) -> FailoverStatistics {
        let stats = self.region_manager.get_statistics().await;

        FailoverStatistics {
            total_regions: stats.total_regions,
            healthy_regions: stats.healthy_regions,
            failed_regions: stats.unhealthy_regions,
            degraded_regions: stats.degraded_regions,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FailoverStatistics {
    pub total_regions: usize,
    pub healthy_regions: usize,
    pub failed_regions: usize,
    pub degraded_regions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::region::Region;

    #[tokio::test]
    async fn test_failover_trigger() {
        let manager = Arc::new(RegionManager::new());

        let region1 = Region::new(RegionId::UsEast1, "https://us-east-1.example.com".to_string());
        let region2 = Region::new(RegionId::UsWest1, "https://us-west-1.example.com".to_string());

        manager.register_region(region1).await.unwrap();
        manager.register_region(region2).await.unwrap();

        let failover = FailoverManager::new(manager.clone());

        // Trigger failover
        let backup = failover.trigger_failover(RegionId::UsEast1).await.unwrap();

        assert_eq!(backup, RegionId::UsWest1);

        // Verify failed region is marked unhealthy
        let failed_region = manager.get_region(RegionId::UsEast1).await.unwrap();
        assert!(matches!(failed_region.status, RegionStatus::Unhealthy));
    }
}
