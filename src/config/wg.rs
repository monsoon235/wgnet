use std::collections::HashMap;
use std::io;
use serde::{Serialize, Deserialize};
use ipnet::IpNet;
use std::net::{AddrParseError, IpAddr, SocketAddr, ToSocketAddrs};
use std::str::{FromStr, Split};
use tonic;

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// pub enum HostEnum {
//     Ip(IpAddr),
//     Hostname(String),
// }
//
// impl FromStr for HostEnum {
//     type Err = io::Error;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         Ok(match IpAddr::from_str(s) {
//             Ok(ip) => Self::Ip(ip),
//             Err(_) => Self::Hostname(s.to_string()),
//         })
//     }
// }
//
// impl ToString for HostEnum {
//     fn to_string(&self) -> String {
//         match self {
//             Self::Ip(ip) => ip.to_string(),
//             Self::Hostname(h) => h.clone(),
//         }
//     }
// }
//
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// pub struct Endpoint {
//     host: HostEnum,
//     port: u16,
// }
//
// impl FromStr for Endpoint {
//     type Err = io::Error;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         todo!()
//     }
// }
//
// impl ToString for Endpoint {
//     fn to_string(&self) -> String {
//         format!("{}:{}", self.host.to_string(), self.port)
//     }
// }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InterfaceConfig {
    pub name: String,
    pub private_key: String,
    pub addrs: Vec<IpNet>,
    pub listen_port: Option<u16>,
    pub mtu: Option<u32>,
    pub internal_endpoint: Option<SocketAddr>,
    pub external_endpoint: Option<SocketAddr>,
    // pub peers: Vec<PeerConfig>,
    pub peers: HashMap<String, PeerConfig>,  // name: peer
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PeerConfig {
    pub public_key: String,
    pub endpoint: Option<SocketAddr>,
    pub allowed_ips: Vec<IpNet>,
    pub preshared_key: Option<String>,
    pub persistent_keepalive: Option<u16>,
}

/// converting between gRPC config

use crate::api::proto;

impl InterfaceConfig {
    pub fn from_proto_config(config: &proto::InterfaceConfig) -> Result<Self, io::Error> {
        let c = InterfaceConfig {
            name: config.name.clone(),
            private_key: config.private_key.clone(),
            addrs: config.addrs.iter().map(|a| IpNet::from_str(&a).unwrap()).collect(),
            listen_port: config.listen_port.map(|p| p as u16),
            mtu: config.mtu,
            internal_endpoint: config.internal_endpoint.as_ref().map(|e| SocketAddr::from_str(&e).unwrap()),
            external_endpoint: config.external_endpoint.as_ref().map(|e| SocketAddr::from_str(&e).unwrap()),
            // peers: config.peers.iter().map(|p| PeerConfig::from_proto_peer(p).unwrap()).collect(),
            peers: config.peers.iter().map(|(k, v)| { (k.clone(), PeerConfig::from_proto_peer(v).unwrap()) }).collect(),
        };
        Ok(c)
    }

    pub fn to_proto_config(&self) -> Result<proto::InterfaceConfig, io::Error> {
        let c = proto::InterfaceConfig {
            name: self.name.clone(),
            private_key: self.private_key.clone(),
            addrs: self.addrs.iter().map(|addr| addr.to_string()).collect(),
            listen_port: self.listen_port.map(|p| p as u32),
            mtu: self.mtu,
            internal_endpoint: self.internal_endpoint.map(|e| e.to_string()),
            external_endpoint: self.external_endpoint.map(|e| e.to_string()),
            // peers: self.peers.iter().map(|p| p.to_proto_peer().unwrap()).collect(),
            peers: self.peers.iter().map(|(k, v)| { (k.clone(), v.to_proto_peer().unwrap()) }).collect(),
        };
        Ok(c)
    }
}

impl PeerConfig {
    pub fn from_proto_peer(config: &proto::PeerConfig) -> Result<Self, io::Error> {
        let c = PeerConfig {
            public_key: config.public_key.clone(),
            endpoint: config.endpoint.as_ref().map(|e| SocketAddr::from_str(&e).unwrap()),
            allowed_ips: config.allowed_ips.iter().map(|a| IpNet::from_str(&a).unwrap()).collect(),
            preshared_key: config.preshared_key.clone(),
            persistent_keepalive: config.persistent_keepalive.map(|p| p as u16),
        };
        Ok(c)
    }

    pub fn to_proto_peer(&self) -> Result<proto::PeerConfig, io::Error> {
        let c = proto::PeerConfig {
            public_key: self.public_key.clone(),
            endpoint: self.endpoint.map(|e| e.to_string()),
            allowed_ips: self.allowed_ips.iter().map(|addr| addr.to_string()).collect(),
            preshared_key: self.preshared_key.clone(),
            persistent_keepalive: self.persistent_keepalive.map(|p| p as u32),
        };
        Ok(c)
    }
}
