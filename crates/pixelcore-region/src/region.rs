use serde::{Deserialize, Serialize};
use std::fmt;

/// Geographic region identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegionId {
    /// North America - US East (Virginia)
    UsEast1,
    /// North America - US West (California)
    UsWest1,
    /// Europe - Ireland
    EuWest1,
    /// Europe - Frankfurt
    EuCentral1,
    /// Asia Pacific - Tokyo
    ApNortheast1,
    /// Asia Pacific - Singapore
    ApSoutheast1,
    /// Asia Pacific - Sydney
    ApSoutheast2,
    /// South America - São Paulo
    SaEast1,
    /// China - Beijing
    CnNorth1,
    /// China - Shanghai
    CnEast1,
    /// China - Shenzhen
    CnSouth1,
    /// China - Chengdu
    CnSouthwest1,
    /// China - Hong Kong
    CnHongKong1,
}

impl RegionId {
    /// Get all available regions
    pub fn all() -> Vec<RegionId> {
        vec![
            RegionId::UsEast1,
            RegionId::UsWest1,
            RegionId::EuWest1,
            RegionId::EuCentral1,
            RegionId::ApNortheast1,
            RegionId::ApSoutheast1,
            RegionId::ApSoutheast2,
            RegionId::SaEast1,
            RegionId::CnNorth1,
            RegionId::CnEast1,
            RegionId::CnSouth1,
            RegionId::CnSouthwest1,
            RegionId::CnHongKong1,
        ]
    }

    /// Get region code (e.g., "us-east-1")
    pub fn code(&self) -> &'static str {
        match self {
            RegionId::UsEast1 => "us-east-1",
            RegionId::UsWest1 => "us-west-1",
            RegionId::EuWest1 => "eu-west-1",
            RegionId::EuCentral1 => "eu-central-1",
            RegionId::ApNortheast1 => "ap-northeast-1",
            RegionId::ApSoutheast1 => "ap-southeast-1",
            RegionId::ApSoutheast2 => "ap-southeast-2",
            RegionId::SaEast1 => "sa-east-1",
            RegionId::CnNorth1 => "cn-north-1",
            RegionId::CnEast1 => "cn-east-1",
            RegionId::CnSouth1 => "cn-south-1",
            RegionId::CnSouthwest1 => "cn-southwest-1",
            RegionId::CnHongKong1 => "cn-hongkong-1",
        }
    }

    /// Get region name
    pub fn name(&self) -> &'static str {
        match self {
            RegionId::UsEast1 => "US East (Virginia)",
            RegionId::UsWest1 => "US West (California)",
            RegionId::EuWest1 => "EU West (Ireland)",
            RegionId::EuCentral1 => "EU Central (Frankfurt)",
            RegionId::ApNortheast1 => "Asia Pacific (Tokyo)",
            RegionId::ApSoutheast1 => "Asia Pacific (Singapore)",
            RegionId::ApSoutheast2 => "Asia Pacific (Sydney)",
            RegionId::SaEast1 => "South America (São Paulo)",
            RegionId::CnNorth1 => "China North (Beijing)",
            RegionId::CnEast1 => "China East (Shanghai)",
            RegionId::CnSouth1 => "China South (Shenzhen)",
            RegionId::CnSouthwest1 => "China Southwest (Chengdu)",
            RegionId::CnHongKong1 => "China (Hong Kong)",
        }
    }

    /// Get approximate latitude and longitude
    pub fn coordinates(&self) -> (f64, f64) {
        match self {
            RegionId::UsEast1 => (37.4316, -78.6569),      // Virginia
            RegionId::UsWest1 => (37.3541, -121.9552),     // California
            RegionId::EuWest1 => (53.3498, -6.2603),       // Ireland
            RegionId::EuCentral1 => (50.1109, 8.6821),     // Frankfurt
            RegionId::ApNortheast1 => (35.6762, 139.6503), // Tokyo
            RegionId::ApSoutheast1 => (1.3521, 103.8198),  // Singapore
            RegionId::ApSoutheast2 => (-33.8688, 151.2093), // Sydney
            RegionId::SaEast1 => (-23.5505, -46.6333),     // São Paulo
            RegionId::CnNorth1 => (39.9042, 116.4074),     // Beijing
            RegionId::CnEast1 => (31.2304, 121.4737),      // Shanghai
            RegionId::CnSouth1 => (22.5431, 114.0579),     // Shenzhen
            RegionId::CnSouthwest1 => (30.5728, 104.0668), // Chengdu
            RegionId::CnHongKong1 => (22.3193, 114.1694),  // Hong Kong
        }
    }

    /// Parse region from code
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "us-east-1" => Some(RegionId::UsEast1),
            "us-west-1" => Some(RegionId::UsWest1),
            "eu-west-1" => Some(RegionId::EuWest1),
            "eu-central-1" => Some(RegionId::EuCentral1),
            "ap-northeast-1" => Some(RegionId::ApNortheast1),
            "ap-southeast-1" => Some(RegionId::ApSoutheast1),
            "ap-southeast-2" => Some(RegionId::ApSoutheast2),
            "sa-east-1" => Some(RegionId::SaEast1),
            "cn-north-1" => Some(RegionId::CnNorth1),
            "cn-east-1" => Some(RegionId::CnEast1),
            "cn-south-1" => Some(RegionId::CnSouth1),
            "cn-southwest-1" => Some(RegionId::CnSouthwest1),
            "cn-hongkong-1" => Some(RegionId::CnHongKong1),
            _ => None,
        }
    }
}

impl fmt::Display for RegionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Region metadata and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: RegionId,
    pub endpoint: String,
    pub status: RegionStatus,
    pub latency_ms: Option<u64>,
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
    pub capacity: RegionCapacity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegionStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionCapacity {
    pub current_load: f64,  // 0.0 to 1.0
    pub max_connections: u32,
    pub active_connections: u32,
}

impl Region {
    pub fn new(id: RegionId, endpoint: String) -> Self {
        Self {
            id,
            endpoint,
            status: RegionStatus::Healthy,
            latency_ms: None,
            last_health_check: None,
            capacity: RegionCapacity {
                current_load: 0.0,
                max_connections: 10000,
                active_connections: 0,
            },
        }
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.status, RegionStatus::Healthy)
    }

    pub fn is_available(&self) -> bool {
        matches!(self.status, RegionStatus::Healthy | RegionStatus::Degraded)
    }

    pub fn load_percentage(&self) -> f64 {
        self.capacity.current_load * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_codes() {
        assert_eq!(RegionId::UsEast1.code(), "us-east-1");
        assert_eq!(RegionId::EuWest1.code(), "eu-west-1");
        assert_eq!(RegionId::ApNortheast1.code(), "ap-northeast-1");
    }

    #[test]
    fn test_region_from_code() {
        assert_eq!(RegionId::from_code("us-east-1"), Some(RegionId::UsEast1));
        assert_eq!(RegionId::from_code("invalid"), None);
    }

    #[test]
    fn test_region_all() {
        let regions = RegionId::all();
        assert_eq!(regions.len(), 13); // 8 original + 5 China regions
    }

    #[test]
    fn test_region_status() {
        let region = Region::new(RegionId::UsEast1, "https://us-east-1.example.com".to_string());
        assert!(region.is_healthy());
        assert!(region.is_available());
    }
}
