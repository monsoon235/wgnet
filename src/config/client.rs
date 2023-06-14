use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};
use wireguard_control::Backend;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ClientConfig {
    pub name: String,
    pub update_interval: u64,
    pub iface_config_dir: String,
    pub backend: String,
}

impl ClientConfig {
    pub fn from_yaml_file(path: &Path) -> Result<Self, io::Error> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)?;
        let mut yaml_str = String::new();
        file.read_to_string(&mut yaml_str)?;
        let config = serde_yaml::from_str(&yaml_str).unwrap();
        Ok(config)
    }

    pub fn to_yaml_file(&self, path: &Path) -> Result<(), io::Error> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path)?;
        let yaml_str = serde_yaml::to_string(&self).unwrap();
        file.write_all(yaml_str.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_client_config() {
        let config = ClientConfig::from_yaml_file(Path::new("example/client.yaml")).unwrap();
        config.to_yaml_file(Path::new("example/tmp/client.yml")).unwrap();
        let config2 = ClientConfig::from_yaml_file(Path::new("example/tmp/client.yml")).unwrap();
        assert_eq!(config, config2);
    }
}
