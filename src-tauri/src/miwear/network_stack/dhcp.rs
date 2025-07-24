use dhcproto::{v4, Decodable, Encodable};
use packet_crafter::{headers::Header, Packet};
use std::{net::Ipv4Addr, sync::Arc};
use crate::{miwear::{packet::{Channel, OpCode}, MiWearDevice}, tools::to_hex_string};

fn compute_checksum(buffer: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut i = 0;
    
    while i + 1 < buffer.len() {
        sum += ((buffer[i] as u32) << 8) | (buffer[i + 1] as u32);
        i += 2;
    }
    
    if i < buffer.len() {
        sum += (buffer[i] as u32) << 8;
    }
    
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    
    (!sum) as u16
}

pub async fn process_dhcp(device: Arc<MiWearDevice>, network_packet: &Vec<u8>, dhcp_process: &mut bool) {
    match Packet::parse(&network_packet) {
        Ok(packet) => {
            if let Some(ip) = packet.get_ip_header() {
                if let Some(udp) = packet.get_udp_header() {
                    if *udp.get_src_port() == 68 || *udp.get_dst_port() == 67 {
                        log::info!("[Dhcp] Start dhcp process...");

                        // ip header length + udp header length
                        let data_offset = (ip.get_length() + 8) as usize;
                        let dhcp_packet = &network_packet[data_offset..];

                        #[cfg(debug_assertions)] {
                            log::info!("[Dhcp] dhcp packet: {}", to_hex_string(dhcp_packet));
                        }

                        if let Ok(dhcp_msg) =
                            v4::Message::decode(&mut v4::Decoder::new(&dhcp_packet.to_vec()))
                        {
                            log::info!("[Dhcp] dhcp message: {:?}", dhcp_msg);

                            match dhcp_msg.opcode() {
                                v4::Opcode::BootRequest => {
                                    let mut dhcp_reply = dhcp_msg.clone();

                                    dhcp_reply.set_opcode(v4::Opcode::BootReply);
                                    dhcp_reply.set_secs(0);
                                    dhcp_reply.set_flags(0.into());
                                    dhcp_reply.set_ciaddr(Ipv4Addr::new(0, 0, 0, 0));
                                    dhcp_reply.set_yiaddr(Ipv4Addr::new(10, 1, 10, 2));
                                    dhcp_reply.set_siaddr(Ipv4Addr::new(10, 1, 10, 1));
                                    dhcp_reply.set_giaddr(Ipv4Addr::new(0, 0, 0, 0));

                                    let mut opts = v4::DhcpOptions::new();

                                    opts.clear();

                                    /*
                                        DHCP option 53: DHCP Offer/Ack
                                        DHCP option 1: 255.255.255.0 subnet mask
                                        DHCP option 3: 10.1.10.1 router
                                        DHCP option 51: 269352960s IP address lease time
                                        DHCP option 54: 10.1.10.1 DHCP server
                                        DHCP option 6: DNS servers 114.114.114.114
                                    */

                                    let req_opts = dhcp_msg.opts();

                                    if let Some(opt_type) =
                                        req_opts.get(v4::OptionCode::MessageType)
                                    {
                                        if let v4::DhcpOption::MessageType(req_type) = opt_type {
                                            if *req_type == v4::MessageType::Discover {
                                                opts.insert(v4::DhcpOption::MessageType(
                                                    v4::MessageType::Offer,
                                                ));
                                            }
                                            if *req_type == v4::MessageType::Request {
                                                opts.insert(v4::DhcpOption::MessageType(
                                                    v4::MessageType::Ack,
                                                ));
                                            }
                                        }
                                    }

                                    opts.insert(v4::DhcpOption::SubnetMask(Ipv4Addr::new(
                                        255, 255, 255, 0,
                                    )));
                                    opts.insert(v4::DhcpOption::Router(vec![Ipv4Addr::new(
                                        10, 1, 10, 1,
                                    )
                                    .clone()]));
                                    opts.insert(v4::DhcpOption::AddressLeaseTime(269352960));
                                    opts.insert(v4::DhcpOption::ServerIdentifier(Ipv4Addr::new(
                                        10, 1, 10, 1,
                                    )));

                                    // 不提供dns服务器让其自动选择
                                    // opts.insert(v4::DhcpOption::DomainNameServer(vec![Ipv4Addr::new(114, 114, 114, 114).clone()]));

                                    // end标记将由库自动处理
                                    // https://github.com/bluecatengineering/dhcproto/issues/75
                                    // opts.insert(v4::DhcpOption::End);

                                    dhcp_reply.set_opts(opts);

                                    log::info!("[Dhcp] dhcp reply message: {:?}", dhcp_reply);

                                    let mut buf = Vec::new();
                                    let mut encoder = v4::Encoder::new(&mut buf);

                                    match dhcp_reply.encode(&mut encoder) {
                                        Ok(_) => {
                                            let udp_len = (buf.len() + 8) as u16;
                                            let ip_len = (udp_len + 20) as u16;

                                            let src_addr = Ipv4Addr::new(255, 255, 255, 255);
                                            let dst_addr = Ipv4Addr::new(10, 1, 10, 1);

                                            let mut udp_header = Vec::new();
                                            // udp源端口
                                            udp_header.extend((0x43 as u16).to_be_bytes());
                                            // udp目的端口
                                            udp_header.extend((0x44 as u16).to_be_bytes());
                                            // udp数据包长度
                                            udp_header.extend(udp_len.to_be_bytes());

                                            // udp伪首部
                                            let mut udp_fake_header = Vec::new();
                                            // udp源地址
                                            udp_fake_header.extend(src_addr.octets());
                                            // udp目的地址
                                            udp_fake_header.extend(dst_addr.octets());
                                            // udp保留字段
                                            udp_fake_header.extend((0x00 as u8).to_be_bytes());
                                            // udp协议类型
                                            udp_fake_header.extend((0x11 as u8).to_be_bytes());
                                            // udp数据包长度
                                            udp_fake_header.extend(udp_len.to_be_bytes());
                                            // 拼接udp报头
                                            udp_fake_header.extend(udp_header.clone());
                                            // 校验和为0x00填充
                                            udp_fake_header.extend((0x00 as u16).to_be_bytes());
                                            // 拼接dhcp数据包
                                            udp_fake_header.extend(buf.clone());

                                            // 计算校验和
                                            let udp_checksum =
                                                compute_checksum(&udp_fake_header) as u16;

                                            udp_header.extend(udp_checksum.to_be_bytes());
                                            udp_header.extend(buf);

                                            let mut ip_header = Vec::new();
                                            // ip数据包版本
                                            ip_header.extend((0x45 as u8).to_be_bytes());
                                            // ip数据包服务类型
                                            ip_header.extend((0x00 as u8).to_be_bytes());
                                            // ip数据包长度
                                            ip_header.extend(ip_len.to_be_bytes());
                                            // ip数据包标识
                                            ip_header.extend((0 as u16).to_be_bytes());
                                            // ip数据包flags
                                            ip_header.extend((0 as u16).to_be_bytes());
                                            // ip数据包生存时间
                                            ip_header.extend((0x40 as u8).to_be_bytes());
                                            // ip数据包协议类型(udp 0x11)
                                            ip_header.extend((0x11 as u8).to_be_bytes());

                                            let mut ip_header_checksum = Vec::new();
                                            ip_header_checksum.extend(ip_header.clone());
                                            // 填充校验和为0x00
                                            ip_header_checksum.extend((0 as u16).to_be_bytes());
                                            // ip数据包源地址
                                            ip_header_checksum.extend(src_addr.octets());
                                            // ip数据包目的地址
                                            ip_header_checksum.extend(dst_addr.octets());

                                            let ip_checksum = compute_checksum(&ip_header_checksum);

                                            // ip数据包校验和
                                            ip_header.extend(ip_checksum.to_be_bytes());
                                            // ip数据包源地址
                                            ip_header.extend(src_addr.octets());
                                            // ip数据包目的地址
                                            ip_header.extend(dst_addr.octets());
                                            // 拼接udp数据包
                                            ip_header.extend(udp_header);

                                            #[cfg(debug_assertions)] {
                                                log::info!(
                                                    "[Dhcp] dhcp reply packet: {}",
                                                    to_hex_string(&ip_header)
                                                );
                                            }

                                            if let Err(err) = device
                                                .send_miwear_pkt(
                                                    Channel::NetWork,
                                                    OpCode::Plain,
                                                    &ip_header,
                                                )
                                                .await
                                            {
                                                log::error!(
                                                    "[Dhcp] Error sending dhcp packet: {}",
                                                    err,
                                                );
                                            }

                                            *dhcp_process = true
                                        }
                                        Err(err) => {
                                            log::error!("[Dhcp] Reply encode fail! err: {}", err);
                                        }
                                    }
                                }
                                v4::Opcode::BootReply => {}
                                v4::Opcode::Unknown(err) => {
                                    log::error!("[Dhcp] Unknown dhcp request err: {}", err);
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(err) => {
            log::warn!("[Dhcp] Failed to parse packet! err: {}", err);
        }
    }
}
