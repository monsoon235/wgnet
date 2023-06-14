use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::path::Path;
use serde::{Serialize, Deserialize, Serializer};
use wireguard_control::Backend;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ServerConfig {
    pub listen: SocketAddr,
    pub iface_config_path: String,
    pub backend: String,
}

impl ServerConfig {
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
    fn test_server_config() {
        let config = ServerConfig::from_yaml_file(Path::new("example/server.yaml")).unwrap();
        config.to_yaml_file(Path::new("example/tmp/server.yml")).unwrap();
        let config2 = ServerConfig::from_yaml_file(Path::new("example/tmp/server.yml")).unwrap();
        assert_eq!(config, config2);
    }
}
