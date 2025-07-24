use std::sync::Arc;

use anyhow::Result;

use crate::tools::hex_stream_to_bytes;

use super::MiWearDevice;

pub async fn hello_packet(device: Arc<MiWearDevice>) -> Result<()> {
    let pkt = hex_stream_to_bytes("badcfe00c00300000100ef").unwrap();
    device.send(pkt).await?;

    Ok(())
}

pub async fn session_config_packet(device: Arc<MiWearDevice>) -> Result<()> {
    let pkt = hex_stream_to_bytes("a5a5020016001d4d0101030001000002020000fc03020020000402001027")
        .unwrap();
    device.send(pkt).await?;

    Ok(())
}
