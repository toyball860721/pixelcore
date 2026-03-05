use crate::{error::Result, region::{Region, RegionId, RegionStatus}};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Region manager for multi-region deployment
pub struct RegionManager {
    regions: Arc<RwLock<HashMap<RegionId, Region>>>,
    health_check_interval: std::time::Duration,
}

impl RegionManager {
    pub fn new() -> Self {
        Self {
            regions: Arc::new(RwLock::new(HashMap::new())),
            health_check_interval: std::time::Duration::from_secs(30),
        }
    }

    /// Register a new region
    pub async fn register_region(&self, region: Region) -> Result<()> {
        let mut regions = self.regions.write().await;

        if regions.contains_key(&region.id) {
            return Err(crate::error::RegionError::RegionAlreadyExists(
                region.id.to_string()
            ));
        }

        info!("Registering region: {} at {}", region.id, region.endpoint);
        regions.insert(region.id, region);
        Ok(())
    }

    /// Unregister a region
    pub async fn unregister_region(&self, region_id: RegionId) -> Result<()> {
        let mut regions = self.regions.write().await;

        regions.remove(&region_id)
            .ok_or_else(|| crate::error::RegionError::RegionNotFound(region_id.to_string()))?;

        info!("Unregistered region: {}", region_id);
        Ok(())
    }

    /// Get a specific region
    pub async fn get_region(&self, region_id: RegionId) -> Result<Region> {
        let regions = self.regions.read().await;
        regions.get(&region_id)
            .cloned()
            .ok_or_else(|| crate::error::RegionError::RegionNotFound(region_id.to_string()))
    }

    /// Get all regions
    pub async fn get_all_regions(&self) -> Vec<Region> {
        let regions = self.regions.read().await;
        regions.values().cloned().collect()
    }

    /// Get all healthy regions
    pub async fn get_healthy_regions(&self) -> Vec<Region> {
        let regions = self.regions.read().await;
        regions.values()
            .filter(|r| r.is_healthy())
            .cloned()
            .collect()
    }

    /// Get all available regions (healthy or degraded)
    pub async fn get_available_regions(&self) -> Vec<Region> {
        let regions = self.regions.read().await;
        regions.values()
            .filter(|r| r.is_available())
            .cloned()
            .collect()
    }

    /// Update region status
    pub async fn update_region_status(&self, region_id: RegionId, status: RegionStatus) -> Result<()> {
        let mut regions = self.regions.write().await;

        let region = regions.get_mut(&region_id)
            .ok_or_else(|| crate::error::RegionError::RegionNotFound(region_id.to_string()))?;

        region.status = status;
        region.last_health_check = Some(chrono::Utc::now());

        info!("Updated region {} status to {:?}", region_id, status);
        Ok(())
    }

    /// Update region latency
    pub async fn update_region_latency(&self, region_id: RegionId, latency_ms: u64) -> Result<()> {
        let mut regions = self.regions.write().await;

        let region = regions.get_mut(&region_id)
            .ok_or_else(|| crate::error::RegionError::RegionNotFound(region_id.to_string()))?;

        region.latency_ms = Some(latency_ms);
        Ok(())
    }

    /// Perform health check on all regions
    pub async fn health_check_all(&self) -> Result<()> {
        let regions = self.get_all_regions().await;

        for region in regions {
            match self.health_check_region(&region).await {
                Ok(latency_ms) => {
                    self.update_region_status(region.id, RegionStatus::Healthy).await?;
                    self.update_region_latency(region.id, latency_ms).await?;
                }
                Err(e) => {
                    warn!("Health check failed for region {}: {}", region.id, e);
                    self.update_region_status(region.id, RegionStatus::Unhealthy).await?;
                }
            }
        }

        Ok(())
    }

    /// Health check a single region
    async fn health_check_region(&self, region: &Region) -> Result<u64> {
        let start = std::time::Instant::now();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        let health_url = format!("{}/health", region.endpoint);
        let response = client.get(&health_url).send().await?;

        if response.status().is_success() {
            let latency_ms = start.elapsed().as_millis() as u64;
            Ok(latency_ms)
        } else {
            Err(crate::error::RegionError::RegionUnhealthy(
                format!("HTTP status: {}", response.status())
            ))
        }
    }

    /// Start background health check loop
    pub async fn start_health_check_loop(self: Arc<Self>) {
        let mut interval = tokio::time::interval(self.health_check_interval);

        tokio::spawn(async move {
            loop {
                interval.tick().await;

                if let Err(e) = self.health_check_all().await {
                    warn!("Health check loop error: {}", e);
                }
            }
        });
    }

    /// Get region statistics
    pub async fn get_statistics(&self) -> RegionStatistics {
        let regions = self.get_all_regions().await;

        let total = regions.len();
        let healthy = regions.iter().filter(|r| r.is_healthy()).count();
        let degraded = regions.iter().filter(|r| matches!(r.status, RegionStatus::Degraded)).count();
        let unhealthy = regions.iter().filter(|r| matches!(r.status, RegionStatus::Unhealthy)).count();

        let avg_latency = if !regions.is_empty() {
            let sum: u64 = regions.iter()
                .filter_map(|r| r.latency_ms)
                .sum();
            Some(sum / regions.len() as u64)
        } else {
            None
        };

        RegionStatistics {
            total_regions: total,
            healthy_regions: healthy,
            degraded_regions: degraded,
            unhealthy_regions: unhealthy,
            average_latency_ms: avg_latency,
        }
    }
}

impl Default for RegionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct RegionStatistics {
    pub total_regions: usize,
    pub healthy_regions: usize,
    pub degraded_regions: usize,
    pub unhealthy_regions: usize,
    pub average_latency_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_region() {
        let manager = RegionManager::new();
        let region = Region::new(RegionId::UsEast1, "https://us-east-1.example.com".to_string());

        assert!(manager.register_region(region).await.is_ok());
    }

    #[tokio::test]
    async fn test_duplicate_region() {
        let manager = RegionManager::new();
        let region = Region::new(RegionId::UsEast1, "https://us-east-1.example.com".to_string());

        manager.register_region(region.clone()).await.unwrap();
        assert!(manager.register_region(region).await.is_err());
    }

    #[tokio::test]
    async fn test_get_healthy_regions() {
        let manager = RegionManager::new();

        let region1 = Region::new(RegionId::UsEast1, "https://us-east-1.example.com".to_string());
        let mut region2 = Region::new(RegionId::UsWest1, "https://us-west-1.example.com".to_string());
        region2.status = RegionStatus::Unhealthy;

        manager.register_region(region1).await.unwrap();
        manager.register_region(region2).await.unwrap();

        let healthy = manager.get_healthy_regions().await;
        assert_eq!(healthy.len(), 1);
        assert_eq!(healthy[0].id, RegionId::UsEast1);
    }
}
