use crate::{error::Result, region::{Region, RegionId}};
use std::sync::Arc;

/// Load balancing strategy
#[derive(Debug, Clone, Copy)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Choose region with lowest latency
    LowestLatency,
    /// Choose region with lowest load
    LowestLoad,
    /// Geographic proximity based on coordinates
    Geographic,
    /// Weighted distribution based on capacity
    Weighted,
}

/// Load balancer for distributing requests across regions
pub struct LoadBalancer {
    strategy: LoadBalancingStrategy,
    round_robin_counter: Arc<tokio::sync::Mutex<usize>>,
}

impl LoadBalancer {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self {
            strategy,
            round_robin_counter: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }

    /// Select the best region for a request
    pub async fn select_region(&self, available_regions: &[Region]) -> Result<RegionId> {
        if available_regions.is_empty() {
            return Err(crate::error::RegionError::NoHealthyRegions);
        }

        match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                self.select_round_robin(available_regions).await
            }
            LoadBalancingStrategy::LowestLatency => {
                self.select_lowest_latency(available_regions)
            }
            LoadBalancingStrategy::LowestLoad => {
                self.select_lowest_load(available_regions)
            }
            LoadBalancingStrategy::Geographic => {
                // For now, fallback to lowest latency
                // In production, would use client's geographic location
                self.select_lowest_latency(available_regions)
            }
            LoadBalancingStrategy::Weighted => {
                self.select_weighted(available_regions)
            }
        }
    }

    /// Select region using round-robin
    async fn select_round_robin(&self, regions: &[Region]) -> Result<RegionId> {
        let mut counter = self.round_robin_counter.lock().await;
        let index = *counter % regions.len();
        *counter = (*counter + 1) % regions.len();

        Ok(regions[index].id)
    }

    /// Select region with lowest latency
    fn select_lowest_latency(&self, regions: &[Region]) -> Result<RegionId> {
        regions.iter()
            .filter_map(|r| r.latency_ms.map(|lat| (r.id, lat)))
            .min_by_key(|(_, lat)| *lat)
            .map(|(id, _)| id)
            .ok_or(crate::error::RegionError::NoHealthyRegions)
    }

    /// Select region with lowest load
    fn select_lowest_load(&self, regions: &[Region]) -> Result<RegionId> {
        regions.iter()
            .min_by(|a, b| {
                a.capacity.current_load
                    .partial_cmp(&b.capacity.current_load)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|r| r.id)
            .ok_or(crate::error::RegionError::NoHealthyRegions)
    }

    /// Select region using weighted distribution
    fn select_weighted(&self, regions: &[Region]) -> Result<RegionId> {
        // Calculate weights based on available capacity
        let weights: Vec<f64> = regions.iter()
            .map(|r| 1.0 - r.capacity.current_load)
            .collect();

        let total_weight: f64 = weights.iter().sum();
        if total_weight == 0.0 {
            return self.select_lowest_load(regions);
        }

        // Simple weighted selection (in production, use proper weighted random)
        let mut max_weight = 0.0;
        let mut selected_idx = 0;

        for (idx, &weight) in weights.iter().enumerate() {
            if weight > max_weight {
                max_weight = weight;
                selected_idx = idx;
            }
        }

        Ok(regions[selected_idx].id)
    }

    /// Calculate distance between two geographic coordinates (Haversine formula)
    pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;

        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lat = (lat2 - lat1).to_radians();
        let delta_lon = (lon2 - lon1).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_KM * c
    }

    /// Select nearest region based on geographic coordinates
    pub fn select_nearest_region(
        &self,
        regions: &[Region],
        client_lat: f64,
        client_lon: f64,
    ) -> Result<RegionId> {
        regions.iter()
            .map(|r| {
                let (region_lat, region_lon) = r.id.coordinates();
                let distance = Self::calculate_distance(client_lat, client_lon, region_lat, region_lon);
                (r.id, distance)
            })
            .min_by(|(_, dist_a), (_, dist_b)| {
                dist_a.partial_cmp(dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(id, _)| id)
            .ok_or(crate::error::RegionError::NoHealthyRegions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_round_robin() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::RoundRobin);

        let regions = vec![
            Region::new(RegionId::UsEast1, "https://us-east-1.example.com".to_string()),
            Region::new(RegionId::UsWest1, "https://us-west-1.example.com".to_string()),
        ];

        let first = lb.select_region(&regions).await.unwrap();
        let second = lb.select_region(&regions).await.unwrap();

        assert_ne!(first, second);
    }

    #[test]
    fn test_lowest_latency() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::LowestLatency);

        let mut region1 = Region::new(RegionId::UsEast1, "https://us-east-1.example.com".to_string());
        region1.latency_ms = Some(50);

        let mut region2 = Region::new(RegionId::UsWest1, "https://us-west-1.example.com".to_string());
        region2.latency_ms = Some(100);

        let regions = vec![region1, region2];
        let selected = lb.select_lowest_latency(&regions).unwrap();

        assert_eq!(selected, RegionId::UsEast1);
    }

    #[test]
    fn test_lowest_load() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::LowestLoad);

        let mut region1 = Region::new(RegionId::UsEast1, "https://us-east-1.example.com".to_string());
        region1.capacity.current_load = 0.8;

        let mut region2 = Region::new(RegionId::UsWest1, "https://us-west-1.example.com".to_string());
        region2.capacity.current_load = 0.3;

        let regions = vec![region1, region2];
        let selected = lb.select_lowest_load(&regions).unwrap();

        assert_eq!(selected, RegionId::UsWest1);
    }

    #[test]
    fn test_distance_calculation() {
        // Distance between New York and Los Angeles
        let distance = LoadBalancer::calculate_distance(40.7128, -74.0060, 34.0522, -118.2437);
        assert!((distance - 3944.0).abs() < 50.0); // Approximately 3944 km
    }

    #[test]
    fn test_nearest_region() {
        let lb = LoadBalancer::new(LoadBalancingStrategy::Geographic);

        let regions = vec![
            Region::new(RegionId::UsEast1, "https://us-east-1.example.com".to_string()),
            Region::new(RegionId::EuWest1, "https://eu-west-1.example.com".to_string()),
        ];

        // Client in New York (closer to US East)
        let selected = lb.select_nearest_region(&regions, 40.7128, -74.0060).unwrap();
        assert_eq!(selected, RegionId::UsEast1);
    }
}
