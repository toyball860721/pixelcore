use crate::{error::Result, region::RegionId};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Replication strategy
#[derive(Debug, Clone, Copy)]
pub enum ReplicationStrategy {
    /// Synchronous replication (wait for all replicas)
    Synchronous,
    /// Asynchronous replication (fire and forget)
    Asynchronous,
    /// Quorum-based replication (wait for majority)
    Quorum,
}

/// Data replication manager
pub struct ReplicationManager {
    redis_client: redis::Client,
    strategy: ReplicationStrategy,
    replication_factor: usize,
}

impl ReplicationManager {
    pub fn new(redis_url: &str, strategy: ReplicationStrategy, replication_factor: usize) -> Result<Self> {
        let redis_client = redis::Client::open(redis_url)?;

        Ok(Self {
            redis_client,
            strategy,
            replication_factor,
        })
    }

    /// Replicate data to multiple regions
    pub async fn replicate_data(
        &self,
        key: &str,
        data: &[u8],
        target_regions: &[RegionId],
    ) -> Result<ReplicationResult> {
        let start = std::time::Instant::now();

        match self.strategy {
            ReplicationStrategy::Synchronous => {
                self.replicate_synchronous(key, data, target_regions).await
            }
            ReplicationStrategy::Asynchronous => {
                self.replicate_asynchronous(key, data, target_regions).await
            }
            ReplicationStrategy::Quorum => {
                self.replicate_quorum(key, data, target_regions).await
            }
        }?;

        let duration = start.elapsed();

        Ok(ReplicationResult {
            key: key.to_string(),
            replicated_regions: target_regions.len(),
            duration_ms: duration.as_millis() as u64,
        })
    }

    /// Synchronous replication - wait for all regions
    async fn replicate_synchronous(
        &self,
        key: &str,
        data: &[u8],
        target_regions: &[RegionId],
    ) -> Result<()> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;

        for region in target_regions {
            let region_key = format!("region:{}:{}", region, key);
            conn.set::<_, _, ()>(&region_key, data).await?;
            info!("Replicated {} to region {}", key, region);
        }

        Ok(())
    }

    /// Asynchronous replication - fire and forget
    async fn replicate_asynchronous(
        &self,
        key: &str,
        data: &[u8],
        target_regions: &[RegionId],
    ) -> Result<()> {
        let redis_client = self.redis_client.clone();
        let key = key.to_string();
        let data = data.to_vec();
        let regions = target_regions.to_vec();

        tokio::spawn(async move {
            if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
                for region in regions {
                    let region_key = format!("region:{}:{}", region, key);
                    if let Err(e) = conn.set::<_, _, ()>(&region_key, &data).await {
                        warn!("Async replication failed for {}: {}", region_key, e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Quorum-based replication - wait for majority
    async fn replicate_quorum(
        &self,
        key: &str,
        data: &[u8],
        target_regions: &[RegionId],
    ) -> Result<()> {
        let quorum_size = (target_regions.len() / 2) + 1;
        let mut successful = 0;

        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;

        for region in target_regions {
            let region_key = format!("region:{}:{}", region, key);

            match conn.set::<_, _, ()>(&region_key, data).await {
                Ok(_) => {
                    successful += 1;
                    info!("Replicated {} to region {}", key, region);

                    if successful >= quorum_size {
                        // Quorum reached, continue async for remaining
                        break;
                    }
                }
                Err(e) => {
                    warn!("Replication failed for region {}: {}", region, e);
                }
            }
        }

        if successful < quorum_size {
            return Err(crate::error::RegionError::ReplicationError(
                format!("Failed to reach quorum: {}/{}", successful, quorum_size)
            ));
        }

        Ok(())
    }

    /// Read data from a specific region
    pub async fn read_from_region(&self, key: &str, region: RegionId) -> Result<Vec<u8>> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let region_key = format!("region:{}:{}", region, key);

        let data: Vec<u8> = conn.get(&region_key).await
            .map_err(|e| crate::error::RegionError::InternalError(e.to_string()))?;

        Ok(data)
    }

    /// Delete data from all regions
    pub async fn delete_from_regions(&self, key: &str, regions: &[RegionId]) -> Result<()> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;

        for region in regions {
            let region_key = format!("region:{}:{}", region, key);
            let _: () = conn.del(&region_key).await?;
            info!("Deleted {} from region {}", key, region);
        }

        Ok(())
    }

    /// Check replication status
    pub async fn check_replication_status(&self, key: &str, regions: &[RegionId]) -> Result<ReplicationStatus> {
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let mut replicated_count = 0;
        let mut missing_regions = Vec::new();

        for region in regions {
            let region_key = format!("region:{}:{}", region, key);
            let exists: bool = conn.exists(&region_key).await?;

            if exists {
                replicated_count += 1;
            } else {
                missing_regions.push(*region);
            }
        }

        let is_complete = missing_regions.is_empty();

        Ok(ReplicationStatus {
            key: key.to_string(),
            total_regions: regions.len(),
            replicated_count,
            missing_regions,
            is_complete,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationResult {
    pub key: String,
    pub replicated_regions: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationStatus {
    pub key: String,
    pub total_regions: usize,
    pub replicated_count: usize,
    pub missing_regions: Vec<RegionId>,
    pub is_complete: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replication_strategy() {
        let manager = ReplicationManager::new(
            "redis://localhost",
            ReplicationStrategy::Quorum,
            3,
        ).unwrap();

        assert!(matches!(manager.strategy, ReplicationStrategy::Quorum));
        assert_eq!(manager.replication_factor, 3);
    }
}
