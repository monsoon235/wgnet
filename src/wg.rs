use std::str::FromStr;
use log;
use std::{io, vec};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use clap::builder::Str;
use serde::ser::SerializeStruct;
use ipnet::IpNet;
use log::Level::Trace;
use toml;
use wireguard_control::{Backend, DeviceUpdate, InterfaceName, Key, PeerConfigBuilder};

use crate::utils::{run_command, resolve_tun_name};
use crate::config::wg::{InterfaceConfig, PeerConfig};


pub struct Interface {
    pub config: InterfaceConfig,
    pub is_up: bool,
    pub peers: HashMap<String, Peer>,
    pub backend: Backend,
}

pub struct Peer {
    pub config: PeerConfig,
}

impl Interface {
    pub fn new(config: &InterfaceConfig, backend: Backend) -> Self {
        Interface {
            config: config.clone(),
            is_up: false,
            peers: HashMap::new(),
            backend,
        }
    }

    pub fn up(&mut self) -> Result<(), io::Error> {
        let config = &self.config;
        let mut update = DeviceUpdate::new();
        update = update.set_private_key(Key::from_base64(&config.private_key).unwrap());
        if let Some(port) = config.listen_port {
            update = update.set_listen_port(port);
        }
        for peer in config.peers.values() {
            let mut p = PeerConfigBuilder::new(
                &Key::from_base64(&peer.public_key).unwrap());
            if let Some(endpoint) = &peer.endpoint {
                p = p.set_endpoint(endpoint.clone());
            }
            if let Some(preshared_key) = &peer.preshared_key {
                p = p.set_preshared_key(Key::from_base64(&preshared_key).unwrap());
            }
            if let Some(persistent_keepalive) = peer.persistent_keepalive {
                p = p.set_persistent_keepalive_interval(persistent_keepalive);
            }
            for ips in &peer.allowed_ips {
                p = p.add_allowed_ip(ips.addr(), ips.prefix_len());
            }
            update = update.add_peer(p);
        }
        update.apply(&InterfaceName::from_str(&config.name).unwrap(), self.backend.clone())?;
        self.set_addr()?;
        self.real_up()?;
        self.is_up = true;
        self.add_route()?;
        Ok(())
    }

    pub fn down(&self) -> Result<(), io::Error> {
        panic!("TODO");
    }

    #[cfg(target_os = "linux")]
    fn set_addr(&self) -> Result<(), io::Error> {
        use crate::utils::linux;
        use netlink_packet_route::address;

        for addr_cidr in &self.config.addrs {
            let iface_idx = linux::if_nametoindex(&self.name)?;
            let (family, nlas) = match addr_cidr {
                IpNet::V4(addr4_cidr) => {
                    let addr_bytes = addr4_cidr.addr().octets().to_vec();
                    (
                        libc::AF_INET as u8,
                        vec![
                            address::Nla::Local(addr_bytes.clone()),
                            address::Nla::Address(addr_bytes),
                        ],
                    )
                }
                IpNet::V6(addr6_cidr) => {
                    let addr_bytes = addr6_cidr.addr().octets().to_vec();
                    (
                        libc::AF_INET6 as u8,
                        vec![
                            address::Nla::Address(addr_bytes),
                        ],
                    )
                }
            };
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn set_addr(&self) -> Result<(), io::Error> {
        let tun_name = resolve_tun_name(&self.config.name)?;
        for addr_cidr in &self.config.addrs {
            match addr_cidr {
                IpNet::V4(addr4_cidr) => {
                    run_command(
                        "ifconfig",
                        &vec![
                            tun_name.as_str(),
                            "inet",
                            addr4_cidr.to_string().as_str(),
                            addr4_cidr.addr().to_string().as_str(),
                            "alias",
                        ],
                    )?;
                }
                IpNet::V6(addr6_cidr) => {
                    run_command(
                        "ifconfig",
                        &vec![
                            tun_name.as_str(),
                            "inet6",
                            addr6_cidr.to_string().as_str(),
                            "alias",
                        ],
                    )?;
                }
            }
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn set_addr(&self) -> Result<(), io::Error> {
        panic!("not implemented");
    }

    #[cfg(target_os = "linux")]
    fn add_route(&self) -> Result<(), io::Error> {
        panic!("not implemented");
    }

    #[cfg(target_os = "macos")]
    fn add_route(&self) -> Result<(), io::Error> {
        let tun_name = resolve_tun_name(&self.config.name)?;
        for addr_cidr in &self.config.addrs {
            run_command(
                "route",
                &vec![
                    "-n",
                    "add",
                    match addr_cidr {
                        IpNet::V4(_) => "-inet",
                        IpNet::V6(_) => "-inet6",
                    },
                    addr_cidr.to_string().as_str(),
                    "-interface",
                    tun_name.as_str(),
                ],
            )?;
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn add_route(&self) -> Result<(), io::Error> {
        panic!("not implemented");
    }

    #[cfg(target_os = "linux")]
    fn real_up(&self) -> Result<(), io::Error> {
        panic!("not implemented");
    }

    #[cfg(target_os = "macos")]
    fn real_up(&self) -> Result<(), io::Error> {
        let mtu = self.config.mtu.unwrap_or(1420);
        let tun_name = resolve_tun_name(&self.config.name)?;
        run_command(
            "ifconfig",
            &vec![
                tun_name.as_str(),
                "mtu",
                mtu.to_string().as_str(),
            ],
        )?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn real_up(&self) -> Result<(), io::Error> {
        panic!("not implemented");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {}
}

