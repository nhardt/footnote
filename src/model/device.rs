use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Device {
    pub name: String,
    pub iroh_endpoint_id: String,
}

impl Device {
    pub fn new(name: String, iroh_endpoint_id: String) -> Self {
        Self {
            name,
            iroh_endpoint_id,
        }
    }

    /// load the local device based on files
    pub fn load(name: String, iroh_endpoint_id: String) -> Self {
        Self {
            name,
            iroh_endpoint_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let device = Device::new(
            "laptop".to_string(),
            "68a56c8723ef24e16ff7e477343d547382235a16d455499aa390f3f309a30107".to_string(),
        );

        assert_eq!(device.name, "laptop");
        assert_eq!(
            device.iroh_endpoint_id,
            "68a56c8723ef24e16ff7e477343d547382235a16d455499aa390f3f309a30107"
        );
    }

    #[test]
    fn test_device_serialization() {
        let device = Device::new("phone".to_string(), "abc123".to_string());

        let json = serde_json::to_string(&device).unwrap();
        let deserialized: Device = serde_json::from_str(&json).unwrap();

        assert_eq!(device, deserialized);
    }
}
