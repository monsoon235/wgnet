use std::io;
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use clap::builder::Str;
use wireguard_control::Key;
use map_macro::map;
use crate::config::wg::InterfaceConfig;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct InviteConfig {
    pub iface_config: InterfaceConfig,
    pub server_socket: SocketAddr,
    pub key: String,
}

impl InviteConfig {
    pub fn from_base64_json(base64_str: &String) -> Result<Self, io::Error> {
        let json_u8 = base64::decode(&base64_str).unwrap();
        let json_str = String::from_utf8(json_u8).unwrap();
        let invite_config = serde_json::from_str(&json_str)?;
        return Ok(invite_config);
    }

    pub fn to_base64_json(&self) -> String {
        let json_str = serde_json::to_string(&self).unwrap();
        log::debug!("json_str: {}", json_str);
        let json_u8 = json_str.as_bytes();
        let base64_str = base64::encode(json_u8);
        return base64_str;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::wg::*;

    #[test]
    fn test_invite_config() {
        let config = InviteConfig {
            iface_config: InterfaceConfig {
                name: "wg0".to_string(),
                private_key: Key::generate_private().to_base64(),
                addrs: vec![
                    "10.1.1.1/16".parse().unwrap(),
                    "fd01:1::1/64".parse().unwrap(),
                ],
                listen_port: Some(51820),
                mtu: Some(1420),
                internal_endpoint: Some("192.168.1.2:51820".parse().unwrap()),
                external_endpoint: Some("6.6.6.6:1234".parse().unwrap()),
                peers: map! {
                    "peer1".to_string() => PeerConfig {
                        public_key: Key::generate_private().generate_public().to_base64(),
                        endpoint: Some("1.2.3.4:51820".parse().unwrap()),
                        allowed_ips: vec![
                            "10.1.0.0/16".parse().unwrap(),
                            "fd01::/64".parse().unwrap(),
                        ],
                        preshared_key: None,
                        persistent_keepalive: Some(25),
                    },
                    "peer2".to_string() => PeerConfig {
                        public_key: Key::generate_private().generate_public().to_base64(),
                        endpoint: Some("1.2.3.5:51820".parse().unwrap()),
                        allowed_ips: vec![
                            "10.2.0.0/16".parse().unwrap(),
                            "fd02::/64".parse().unwrap(),
                        ],
                        preshared_key: None,
                        persistent_keepalive: Some(25),
                    }
                },
            },
            server_socket: "10.1.0.0:8888".parse().unwrap(),
            key: "invite_key".to_string(),
        };
        let base64_str = config.to_base64_json();
        log::debug!("base64_str: {}", base64_str);
        let config2 = InviteConfig::from_base64_json(&base64_str).unwrap();
        assert_eq!(config, config2);
    }
}
