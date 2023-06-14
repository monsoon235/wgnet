use std::process;
use std::io;
use std::str::FromStr;
use log;
use wireguard_control::InterfaceName;

pub fn run_command(cmd: &str, args: &Vec<&str>) -> Result<process::Output, io::Error> {
    log::debug!("run command: {} {}", cmd, args.join(" "));
    let output = std::process::Command::new(cmd).args(args).output()?;
    log::debug!("run command status: {:?}", output.status.code());
    log::debug!("command stdout: {}", String::from_utf8_lossy(&output.stdout));
    log::debug!("command stderr: {}", String::from_utf8_lossy(&output.stderr));
    if output.status.success() {
        Ok(output)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("failed to run {} {}: {}", cmd, args.join(" "), String::from_utf8_lossy(&output.stderr)),
        ))
    }
}

pub fn resolve_tun_name(name: &String) -> Result<String, io::Error> {
    let real_interface = wireguard_control::backends::userspace::resolve_tun(
        &InterfaceName::from_str(&name)?
    )?;
    Ok(real_interface)
}

#[cfg(target_os = "linux")]
pub mod linux {
    use std::fmt::Debug;
    use std::io;
    use wireguard_control::InterfaceName;

    pub fn if_nametoindex(interface: &InterfaceName) -> Result<u32, io::Error> {
        match unsafe { libc::if_nametoindex(interface.as_ptr()) } {
            0 => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("couldn't find interface '{}'.", interface),
            )),
            index => Ok(index),
        }
    }

    pub fn netlink_request_genl<F>(
        mut message: GenlMessage<F>,
        flags: Option<u16>,
    ) -> Result<Vec<NetlinkMessage<GenlMessage<F>>>, io::Error>
        where
            F: GenlFamily + Clone + Debug + Eq,
            GenlMessage<F>: Clone + Debug + Eq + NetlinkSerializable + NetlinkDeserializable,
    {
        if message.family_id() == 0 {
            let genlmsg: GenlMessage<GenlCtrl> = GenlMessage::from_payload(GenlCtrl {
                cmd: GenlCtrlCmd::GetFamily,
                nlas: vec![GenlCtrlAttrs::FamilyName(F::family_name().to_string())],
            });
            let responses =
                netlink_request_genl::<GenlCtrl>(genlmsg, Some(NLM_F_REQUEST | NLM_F_ACK))?;

            match responses.get(0) {
                Some(NetlinkMessage {
                         payload:
                         NetlinkPayload::InnerMessage(GenlMessage {
                                                          payload: GenlCtrl { nlas, .. },
                                                          ..
                                                      }),
                         ..
                     }) => {
                    let family_id = get_nla_value!(nlas, GenlCtrlAttrs, FamilyId)
                        .ok_or_else(|| io::ErrorKind::NotFound)?;
                    message.set_resolved_family_id(*family_id);
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unexpected netlink payload",
                    ));
                }
            };
        }
        netlink_request(message, flags, NETLINK_GENERIC)
    }

    pub fn netlink_request_rtnl(
        message: RtnlMessage,
        flags: Option<u16>,
    ) -> Result<Vec<NetlinkMessage<RtnlMessage>>, io::Error> {
        netlink_request(message, flags, NETLINK_ROUTE)
    }

    pub fn netlink_request<I>(
        message: I,
        flags: Option<u16>,
        socket: isize,
    ) -> Result<Vec<NetlinkMessage<I>>, io::Error>
        where
            NetlinkPayload<I>: From<I>,
            I: Clone + Debug + Eq + NetlinkSerializable + NetlinkDeserializable,
    {
        let mut req = NetlinkMessage::from(message);

        if req.buffer_len() > MAX_NETLINK_BUFFER_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Serialized netlink packet ({} bytes) larger than maximum size {}: {:?}",
                    req.buffer_len(),
                    MAX_NETLINK_BUFFER_LENGTH,
                    req
                ),
            ));
        }

        req.header.flags = flags.unwrap_or(NLM_F_REQUEST | NLM_F_ACK | NLM_F_EXCL | NLM_F_CREATE);
        req.finalize();
        let mut buf = [0; MAX_NETLINK_BUFFER_LENGTH];
        req.serialize(&mut buf);
        let len = req.buffer_len();

        let socket = Socket::new(socket)?;
        let kernel_addr = netlink_sys::SocketAddr::new(0, 0);
        socket.connect(&kernel_addr)?;
        let n_sent = socket.send(&buf[..len], 0)?;
        if n_sent != len {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "failed to send netlink request",
            ));
        }

        let mut responses = vec![];
        loop {
            let n_received = socket.recv(&mut &mut buf[..], 0)?;
            let mut offset = 0;
            loop {
                let bytes = &buf[offset..];
                let response = NetlinkMessage::<I>::deserialize(bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                match response.payload {
                    // We've parsed all parts of the response and can leave the loop.
                    NetlinkPayload::Ack(_) | NetlinkPayload::Done => return Ok(responses),
                    NetlinkPayload::Error(e) => return Err(e.into()),
                    _ => {}
                }
                responses.push(response.clone());
                offset += response.header.length as usize;
                if offset == n_received || response.header.length == 0 {
                    // We've fully parsed the datagram, but there may be further datagrams
                    // with additional netlink response parts.
                    break;
                }
            }
        }
    }
}


