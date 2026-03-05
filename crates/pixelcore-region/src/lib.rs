//! PixelCore Multi-region Deployment Module
//!
//! Provides comprehensive multi-region deployment capabilities including:
//! - Region management and health monitoring
//! - Load balancing across regions
//! - Data replication with multiple strategies
//! - Automatic failover and recovery

pub mod error;
pub mod failover;
pub mod load_balancer;
pub mod manager;
pub mod region;
pub mod replication;

pub use error::{RegionError, Result};
pub use failover::{FailoverManager, FailoverStatistics};
pub use load_balancer::{LoadBalancer, LoadBalancingStrategy};
pub use manager::{RegionManager, RegionStatistics};
pub use region::{Region, RegionCapacity, RegionId, RegionStatus};
pub use replication::{ReplicationManager, ReplicationResult, ReplicationStatus, ReplicationStrategy};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify that all public exports are accessible
        let _region_id: Option<RegionId> = None;
        let _manager: Option<RegionManager> = None;
    }
}
