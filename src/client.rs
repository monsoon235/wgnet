use std::collections::HashMap;
use std::io;
use std::io::Error;
use std::iter::Map;
use std::ops::Mul;
use std::path::PathBuf;
use std::time::Duration;
use log::log;
use tokio::time;
use wireguard_control::Backend;
use crate::wg::Interface;
use crate::config::client::ClientConfig;
use crate::config::invite::InviteConfig;
use crate::api::proto;
use crate::config::wg::InterfaceConfig;

pub struct Client {
    config: ClientConfig,
    // name: iface
    ifaces: HashMap<String, Interface>,
    rpc_client: proto::rpc_client::RpcClient<tonic::transport::Channel>,
    exiting: bool,
}

impl Client {
    pub async fn run(&mut self) {
        self.exiting = false;
        for (name, iface) in self.ifaces.iter_mut() {
            log::info!("Interface {} upping ...", name);
            match iface.up() {
                Ok(_) => log::info!("Interface {name} upped successfully"),
                Err(e) => log::error!("Interface {name} upped failed: {e}"),
            }
        }
        // update config and peers periodically
        loop {
            if self.exiting {
                log::info!("Exiting the client ...");
                for (name, mut iface) in self.ifaces.iter() {
                    log::debug!("Interface {name} downing ...");
                    match iface.down() {
                        Ok(_) => log::info!("Interface {name} is down"),
                        Err(e) => log::error!("Interface {name} down failed"),
                    }
                }
                return;
            }
            let names: Vec<String> = self.ifaces.keys().cloned().collect();
            for name in names {
                log::debug!("Interface {name} updating ...");
                match self.update_peers(&name).await {
                    Ok(_) => log::debug!("Interface {name} updated successfully"),
                    Err(e) => log::error!("Interface {name} updated failed: {e}"),
                };
            }
            tokio::time::sleep(Duration::from_secs(self.config.update_interval)).await;
        }
    }

    pub async fn redeem_invite(&mut self, invite: &InviteConfig) -> Result<(), io::Error> {
        // init iface
        #[cfg(target_os = "linux")]
            let backend = match self.config.backend.to_lowercase().as_str() {
            "kernel" => Backend::Kernel,
            "userspace" => Backend::Userspace,
            _ => {
                log::error!("Unknown backend \"{}\", use \"kernel\" by default", self.config.backend);
                Backend::Kernel
            }
        };
        #[cfg(not(target_os = "linux"))]
            let backend = match self.config.backend.to_lowercase().as_str() {
            "userspace" => Backend::Userspace,
            "kernel" => {
                log::error!("\"{}\" backend is not supported in current OS, using \"userspace\" instead", self.config.backend);
                Backend::Userspace
            }
            _ => {
                log::error!("Unknown backend \"{}\", use \"userspace\" by default", self.config.backend);
                Backend::Userspace
            }
        };
        // up the init iface
        let mut iface_init = Interface::new(&invite.iface_config, backend);
        let name = iface_init.config.name.clone();
        iface_init.up().unwrap();
        // build rpc client
        self.rpc_client = proto::rpc_client::RpcClient::connect(invite.server_socket.to_string()).await.unwrap();
        // test rpc client
        let req = proto::PingRequest {
            msg: format!("I'm {}", invite.key),
        };
        let resp = self.rpc_client.ping(req).await.unwrap().into_inner();
        log::debug!("Ping response: {}", resp.msg);
        // redeem invite
        let req = proto::RedeemInviteRequest {
            key: invite.key.clone(),
        };
        let resp = self.rpc_client.redeem_invite(req).await.unwrap().into_inner();
        // down the init iface
        iface_init.down().unwrap();
        // add real ifaces
        for r in resp.iface_config.iter() {
            let iface_config = InterfaceConfig::from_proto_config(r).unwrap();
            let iface = Interface::new(&iface_config, backend);
            self.ifaces.insert(iface_config.name.clone(), iface);
        }
        Ok(())
    }

    pub async fn post_endpoint(&mut self, name: &str) -> Result<(), io::Error> {
        let iface = self.ifaces.get(name).unwrap();
        let req = proto::PostEndpointRequest {
            key: iface.config.private_key.clone(),
            internal_endpoint: iface.config.internal_endpoint.map(|e| { e.to_string() }),
            external_endpoint: iface.config.external_endpoint.map(|e| { e.to_string() }),
        };
        let resp = self.rpc_client.post_endpoint(req).await.unwrap().into_inner();
        if resp.ok {
            log::debug!("Interface {}: post endpoint successfully", name);
        } else {
            log::error!("Post endpoint failed");
        }
        Ok(())
    }

    pub async fn update_peers(&mut self, name: &str) -> Result<(), io::Error> {
        let iface = self.ifaces.get(name).unwrap();
        let req = proto::GetPeersRequest {
            key: iface.config.private_key.clone(),
        };
        let resp = self.rpc_client.get_peers(req).await.unwrap().into_inner();
        println!("Get peers response: {:?}", resp);
        let iface = self.ifaces.get_mut(name).unwrap();
        // iface.update_peers(resp.peers);
        panic!("Not implemented");
        Ok(())
    }

    // 扫描 config.iface_config_dir 目录，找到所有已知的 wg 配置文件
    pub async fn scan_wg_config_dir(&mut self) {
        self.config.iface_config_dir
    }
}
