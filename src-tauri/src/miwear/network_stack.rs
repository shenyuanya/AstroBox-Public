use super::device::MiWearDevice;
use crate::miwear::network_stack::meter::BandwidthMeter;
use crate::miwear::packet::{Channel, OpCode};
use crate::tools::{hex_stream_to_bytes, to_hex_string};
use std::{fs};
use ipstack::{IpNumber, IpStack, IpStackConfig};
use pcap_file::pcap::{PcapPacket, PcapWriter};
use serde::Serialize;
use tauri::{Emitter, Manager};
use std::fs::File;
use std::io::{Error, ErrorKind, Result};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use futures_util::Sink;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::sync::PollSender;
use udp_stream::UdpStream;
use etherparse::Icmpv4Header;
use once_cell::sync::Lazy;
use chrono::Local;

pub mod dhcp;
pub mod meter;

pub const NETWORK_SPEED_EVENT: &str = "network-speed";

static START_TIME: Lazy<String> =
    Lazy::new(|| Local::now().format("%Y-%m-%d_%H-%M-%S").to_string());

#[derive(Serialize, Clone)]
pub struct NetWorkSpeed {
    pub write: f64,
    pub read: f64,
}

pub struct MiWearTunDevice {
    rx: mpsc::Receiver<Vec<u8>>,
    tx_send: PollSender<Vec<u8>>,
    capture: tokio::sync::Mutex<PcapWriter<File>>,
    meter: BandwidthMeter,
}

impl AsyncRead for MiWearTunDevice {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        log::info!("[MiWearTunDevice] poll_read");
        match self.rx.poll_recv(cx) {
            Poll::Ready(Some(packet)) => {
                if packet.len() > buf.remaining() {
                    log::warn!(
                        "[MiWearTunDevice] Received packet ({} bytes) larger than buffer ({} bytes). Truncating.",
                        packet.len(),
                        buf.remaining()
                    );
                    buf.put_slice(&packet[..buf.remaining()]);
                } else {
                    buf.put_slice(&packet);
                }

                self.meter.add_read(packet.len());

                let mut ethernet_header = hex_stream_to_bytes("000000000000a5a5a5a5a5a50800").unwrap();

                ethernet_header.extend(packet.clone());

                self.capture.get_mut().write_packet(&PcapPacket {
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                    orig_len: ethernet_header.len() as u32,
                    data: ethernet_header.into(),
                }).unwrap();

                #[cfg(debug_assertions)] {
                    let data = to_hex_string(&packet);
                    log::info!("[MiWearTunDevice] poll_read buf(size: {}) data: {}", packet.len(), data);
                }

                Poll::Ready(Ok(()))
            }
            Poll::Ready(None) => Poll::Ready(Err(Error::new(
                ErrorKind::BrokenPipe,
                "MiWearTunDevice channel closed",
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for MiWearTunDevice {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        let packet_to_send = buf.to_vec();
        #[cfg(debug_assertions)] {
            let data = to_hex_string(&buf);
            log::info!(
                "[MiWearTunDevice] poll_write: packet_to_send(size: {}) data: {}",
                packet_to_send.len(),
                data
            );
        }

        self.meter.add_written(packet_to_send.len());

        let mut ethernet_header = hex_stream_to_bytes("a5a5a5a5a5a50000000000000800").unwrap();

        ethernet_header.extend(buf);

        self.capture.get_mut().write_packet(&PcapPacket {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            orig_len: ethernet_header.len() as u32,
            data: ethernet_header.into(),
        }).unwrap();

        match Pin::new(&mut self.tx_send).poll_ready(cx) {
            Poll::Ready(Ok(())) => {
                if let Err(_) = Pin::new(&mut self.tx_send).start_send(packet_to_send) {
                    return Poll::Ready(Err(Error::new(ErrorKind::BrokenPipe, "send channel closed")));
                }
                Poll::Ready(Ok(buf.len()))
            }
            Poll::Ready(Err(_)) => Poll::Ready(Err(Error::new(ErrorKind::BrokenPipe, "send channel closed"))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }
}

pub fn start_network_stack(device: Arc<MiWearDevice>) -> mpsc::Sender<Vec<u8>> {
    let (tx, rx) = mpsc::channel::<Vec<u8>>(100);
    let (tx_send, mut rx_send) = mpsc::channel::<Vec<u8>>(100);
    let poll_tx_send = PollSender::new(tx_send.clone());

    let base_path = format!(
        "{}/rslogs",
        crate::APP_HANDLE
            .get()
            .unwrap()
            .path()
            .app_log_dir()
            .unwrap()
            .to_string_lossy()
    );

    fs::create_dir_all(&base_path).expect("Error creating directory");
    let file_path = format!("{}/{}.pcap", base_path, &*START_TIME);

    let file_out = File::create(file_path).expect("Error creating file out");

    let pcap_writer = PcapWriter::new(file_out).expect("Error writing file");

    let tun_device = MiWearTunDevice {
        rx,
        tx_send: poll_tx_send,
        capture: pcap_writer.into(),
        meter: BandwidthMeter::new(Duration::from_secs(5)),
    };


    tokio::spawn({
        /* let device = device.clone(); */
        let meter_clone = tun_device.meter.clone();
        async move {
            let mut disconnect_rx = crate::miwear::subscribe_disconnect();
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                tokio::select! {
                    _ = interval.tick() => {

                        let write_speed = meter_clone.write_speed();
                        let read_speed = meter_clone.read_speed();

                        /* log::info!(
                            "[BandwidthMeter] ↓ {:.2} KiB/s  ↑ {:.2} KiB/s",
                            write_speed/1024.0,
                            read_speed/1024.0,
                        ); */

                        if let Some(app) = crate::APP_HANDLE.get() {
                            let _ = app.emit(NETWORK_SPEED_EVENT, NetWorkSpeed{
                                write: write_speed,
                                read: read_speed,
                            });
                        }

                        /* let mut write_speed_lock = device.network_write_speed.lock().await;
                        *write_speed_lock = write_speed;

                        let mut read_speed_lock = device.network_write_speed.lock().await;
                        *read_speed_lock = read_speed; */

                    }
                    _ = disconnect_rx.recv() => {
                        log::info!("[BandwidthMeter] Device disconnected, stopping network meter");
                        break;
                    }
                }
            }
        }
    });

    tokio::spawn({
        let device = device.clone();
        async move {
            let mut disconnect_rx = crate::miwear::subscribe_disconnect();
            loop {
                tokio::select! {
                    Some(packet) = rx_send.recv() => {
                        log::info!("[MiWearTunDevice] Sending IP packet ({} bytes) over Bluetooth.", packet.len());
                        tokio::select! {
                            _ = disconnect_rx.recv() => {
                                log::info!("[MiWearTunDevice] Aborted send due to disconnect");
                            }
                            res = device.send_miwear_pkt(Channel::NetWork, OpCode::Plain, &packet) => {
                                if let Err(e) = res {
                                    log::error!(
                                        "[MiWearTunDevice] Error sending packet from send_loop: {}",
                                        e
                                    );
                                }
                            }
                        }
                    }
                    else => {
                        break;
                    }
                }
            }
        }
    });

    tokio::spawn(async move {

        let mtu = device.state.read().await.network_mtu;

        let mut disconnect_rx = crate::miwear::subscribe_disconnect();
        let mut config = IpStackConfig::default();
        config.mtu(mtu);

        let mut ip_stack = IpStack::new(config, tun_device);

        log::info!(
            "[IpStack] Network stack started for device {} mtu: {}",
            device.state.read().await.addr,
            mtu,
        );

        let count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let serial_number = std::sync::atomic::AtomicUsize::new(0);

        loop {
            let count = count.clone();
            let number = serial_number.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            tokio::select! {
                _ = disconnect_rx.recv() => {
                    log::info!("[IpStack] Device disconnected, stopping network stack");
                    break;
                }
                accept_res = ip_stack.accept() => {
                    match accept_res {
                        Ok(stream) => match stream {
                            ipstack::IpStackStream::Tcp(mut tcp) => {
                                let mut s = match TcpStream::connect(tcp.peer_addr()).await {
                                    Ok(s) => s,
                                    Err(e) => {
                                        log::info!("connect TCP server failed \"{}\"", e);
                                        continue;
                                    }
                                };
                                let c = count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                                let tcp_number = number;
                                log::info!("#{tcp_number} TCP connecting, session count {c}");
                                tokio::spawn(async move {
                                    if let Err(err) = tokio::io::copy_bidirectional(&mut tcp, &mut s).await
                                    {
                                        log::info!("#{tcp_number} TCP error: {}", err);
                                    }
                                    if let Err(e) = s.shutdown().await {
                                        log::info!("#{tcp_number} TCP upstream shutdown error: {}", e);
                                    }
                                    if let Err(e) = tcp.shutdown().await {
                                        log::info!("#{tcp_number} TCP stack stream shutdown error: {}", e);
                                    }
                                    let c = count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed) - 1;
                                    log::info!("#{tcp_number} TCP closed, session count {c}");
                                });
                            }
                            ipstack::IpStackStream::Udp(mut udp) => {
                                let mut s = match UdpStream::connect(udp.peer_addr()).await {
                                    Ok(s) => s,
                                    Err(e) => {
                                        log::info!("connect UDP server failed \"{}\"", e);
                                        continue;
                                    }
                                };
                                let c = count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                                let udp_number = number;
                                log::info!("#{udp_number} UDP connecting, session count {c}");
                                tokio::spawn(async move {
                                    if let Err(err) = tokio::io::copy_bidirectional(&mut udp, &mut s).await
                                    {
                                        log::info!("#{udp_number} UDP error: {}", err);
                                    }
                                    s.shutdown();
                                    if let Err(e) = udp.shutdown().await {
                                        log::info!("#{udp_number} UDP stack stream shutdown error: {}", e);
                                    }
                                    let c = count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed) - 1;
                                    log::info!("#{udp_number} UDP closed, session count {c}");
                                });
                            }
                            ipstack::IpStackStream::UnknownTransport(u) => {
                                let n = number;
                                if u.src_addr().is_ipv4() && u.ip_protocol() == IpNumber::ICMP {
                                    let (icmp_header, req_payload) = match Icmpv4Header::from_slice(u.payload()) {
                                        Ok(data) => { data },
                                        Err(_) => { continue; },
                                    };
                                    if let etherparse::Icmpv4Type::EchoRequest(echo) = icmp_header.icmp_type
                                    {
                                        log::info!("#{n} ICMPv4 echo");
                                        let mut resp =
                                            Icmpv4Header::new(etherparse::Icmpv4Type::EchoReply(echo));
                                        resp.update_checksum(req_payload);
                                        let mut payload = resp.to_bytes().to_vec();
                                        payload.extend_from_slice(req_payload);
                                        match u.send(payload) {
                                            Ok(_) => {
                                                log::info!("#{n} ICMPv4 send success!");
                                            }
                                            Err(_) => {
                                                log::info!("#{n} ICMPv4 send fail!");
                                            }
                                        }
                                    } else {
                                        log::info!("#{n} ICMPv4");
                                    }
                                    continue;
                                }
                                log::info!("#{n} unknown transport - Ip Protocol {:?}", u.ip_protocol());
                                continue;
                            }
                            ipstack::IpStackStream::UnknownNetwork(pkt) => {
                                log::info!("#{number} unknown network - {} bytes", pkt.len());
                                continue;
                            }
                        },
                        Err(e) => {
                            log::error!(
                                "[IpStack] Network stack on device {} stopped with error: {}",
                                device.state.read().await.addr,
                                e
                            );
                        }
                    }
                }
            }
        }
    });

    tx
}
