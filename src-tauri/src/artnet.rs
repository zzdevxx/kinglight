use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::Arc;

use tokio::sync::{RwLock, mpsc, oneshot};

use crate::engine::FrameBundle;
use crate::model::OutputStatusCore;
use crate::{ART_DMX_LEN, ARTNET_PORT, DMX_LEN, OUTPUT_INTERVAL};

pub const MAX_PORT_ADDRESS: u16 = 0x7fff;

pub struct Sequence(u8);

impl Sequence {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn next(&mut self) -> u8 {
        self.0 = if self.0 >= 255 { 1 } else { self.0 + 1 };
        self.0
    }
}

impl Default for Sequence {
    fn default() -> Self {
        Self::new()
    }
}

pub fn validate_port_address(port_address: u32, compat_zero: bool) -> Result<u16, String> {
    if port_address == 0 {
        if compat_zero {
            return Ok(0);
        }
        return Err("INVALID_PORT_ADDRESS: portAddress 0 memerlukan compatZero true".to_string());
    }
    if port_address > MAX_PORT_ADDRESS as u32 {
        return Err(format!(
            "INVALID_PORT_ADDRESS: portAddress {port_address} melebihi 32767"
        ));
    }
    Ok(port_address as u16)
}

pub fn encode_port_address(port_address: u16) -> (u8, u8) {
    (
        (port_address & 0x00ff) as u8,
        ((port_address >> 8) & 0x007f) as u8,
    )
}

pub fn build_art_dmx(dmx: &[u8; DMX_LEN], port_address: u16, sequence: u8) -> [u8; ART_DMX_LEN] {
    let mut packet = [0u8; ART_DMX_LEN];
    packet[0..8].copy_from_slice(b"Art-Net\0");
    packet[9] = 0x50;
    packet[10] = 0x00;
    packet[11] = 0x0e;
    packet[12] = sequence;
    packet[13] = 0x00;
    let (sub_uni, net) = encode_port_address(port_address);
    packet[14] = sub_uni;
    packet[15] = net;
    packet[16] = 0x02;
    packet[17] = 0x00;
    packet[18..].copy_from_slice(dmx);
    packet
}

pub struct StopFrames {
    pub first: [u8; ART_DMX_LEN],
    pub second: [u8; ART_DMX_LEN],
    pub sequence: u8,
}

pub fn build_stop_frames(
    dmx: &[u8; DMX_LEN],
    port_address: u16,
    sequence: &mut Sequence,
) -> StopFrames {
    let first_sequence = sequence.next();
    let second_sequence = sequence.next();
    StopFrames {
        first: build_art_dmx(dmx, port_address, first_sequence),
        second: build_art_dmx(dmx, port_address, second_sequence),
        sequence: second_sequence,
    }
}

fn resolve_target(target: &str) -> Result<SocketAddr, String> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return Err("INVALID_OUTPUT_CONFIG: target kosong".to_string());
    }
    let with_port = if trimmed.contains(':') && !trimmed.starts_with('[') {
        trimmed.to_string()
    } else {
        format!("{trimmed}:{ARTNET_PORT}")
    };
    with_port
        .to_socket_addrs()
        .map_err(|error| format!("INVALID_OUTPUT_CONFIG: target tidak valid: {error}"))?
        .next()
        .ok_or_else(|| "INVALID_OUTPUT_CONFIG: target tidak memiliki alamat".to_string())
}

enum OutputRequest {
    Configure {
        target: String,
        port_address: u16,
        compat_zero: bool,
        reply: oneshot::Sender<Result<(), String>>,
    },
    Arm {
        reply: oneshot::Sender<Result<(), String>>,
    },
    Stop {
        frame: FrameBundle,
        reply: oneshot::Sender<Result<(), String>>,
    },
    Shutdown,
}

#[derive(Clone)]
pub struct OutputController {
    tx: mpsc::UnboundedSender<OutputRequest>,
    status: Arc<RwLock<OutputStatusCore>>,
}

impl OutputController {
    pub fn start(
        frames: Arc<RwLock<Arc<FrameBundle>>>,
        status: Arc<RwLock<OutputStatusCore>>,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let controller = Self {
            tx,
            status: status.clone(),
        };
        tauri::async_runtime::spawn(output_actor(rx, frames, status));
        controller
    }

    pub async fn status(&self) -> OutputStatusCore {
        self.status.read().await.clone()
    }

    pub async fn configure(
        &self,
        target: String,
        port_address: u16,
        compat_zero: bool,
    ) -> Result<(), String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(OutputRequest::Configure {
                target,
                port_address,
                compat_zero,
                reply: reply_tx,
            })
            .map_err(|_| "OUTPUT_ACTOR_STOPPED".to_string())?;
        reply_rx
            .await
            .map_err(|_| "OUTPUT_ACTOR_STOPPED".to_string())?
    }

    pub async fn arm(&self) -> Result<(), String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(OutputRequest::Arm { reply: reply_tx })
            .map_err(|_| "OUTPUT_ACTOR_STOPPED".to_string())?;
        reply_rx
            .await
            .map_err(|_| "OUTPUT_ACTOR_STOPPED".to_string())?
    }

    pub async fn stop(&self, frame: FrameBundle) -> Result<(), String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(OutputRequest::Stop {
                frame,
                reply: reply_tx,
            })
            .map_err(|_| "OUTPUT_ACTOR_STOPPED".to_string())?;
        reply_rx
            .await
            .map_err(|_| "OUTPUT_ACTOR_STOPPED".to_string())?
    }

    pub fn shutdown(&self) {
        if self.tx.send(OutputRequest::Shutdown).is_err() {
            eprintln!("[kinglight] warn=output_actor_already_stopped");
        }
    }
}

async fn output_actor(
    mut rx: mpsc::UnboundedReceiver<OutputRequest>,
    frames: Arc<RwLock<Arc<FrameBundle>>>,
    status: Arc<RwLock<OutputStatusCore>>,
) {
    let mut socket: Option<UdpSocket> = None;
    let mut destination: Option<SocketAddr> = None;
    let mut sequence = Sequence::new();
    let mut deadline = tokio::time::Instant::now() + OUTPUT_INTERVAL;

    loop {
        tokio::select! {
            request = rx.recv() => {
                match request {
                    Some(OutputRequest::Configure { target, port_address, compat_zero, reply }) => {
                        let result = configure_actor(&status, &mut socket, &mut destination, target, port_address, compat_zero).await;
                        let _ = reply.send(result);
                    }
                    Some(OutputRequest::Arm { reply }) => {
                        let result = arm_actor(&status, &mut socket, &destination, &mut sequence).await;
                        let _ = reply.send(result);
                    }
                    Some(OutputRequest::Stop { frame, reply }) => {
                        let result = stop_actor(&status, &mut socket, &destination, &mut sequence, &frame).await;
                        let _ = reply.send(result);
                    }
                    Some(OutputRequest::Shutdown) | None => break,
                }
            }
            _ = tokio::time::sleep_until(deadline) => {
                if let (Some(sock), Some(addr)) = (socket.as_ref(), destination) {
                    send_current_frame(&frames, &status, sock, addr, &mut sequence).await;
                }
                deadline = tokio::time::Instant::now() + OUTPUT_INTERVAL;
            }
        }
    }
}

async fn configure_actor(
    status: &Arc<RwLock<OutputStatusCore>>,
    socket: &mut Option<UdpSocket>,
    destination: &mut Option<SocketAddr>,
    target: String,
    port_address: u16,
    compat_zero: bool,
) -> Result<(), String> {
    {
        let mut guard = status.write().await;
        if guard.armed {
            return Err("OUTPUT_ARMED: stop output sebelum mengubah konfigurasi".to_string());
        }
        validate_port_address(port_address as u32, compat_zero)?;
        let resolved = resolve_target(&target)?;
        guard.target = target.trim().to_string();
        guard.port_address = port_address;
        guard.compat_zero = compat_zero;
        guard.last_error = None;
        *socket = None;
        *destination = Some(resolved);
    }
    Ok(())
}

async fn arm_actor(
    status: &Arc<RwLock<OutputStatusCore>>,
    socket: &mut Option<UdpSocket>,
    destination: &Option<SocketAddr>,
    sequence: &mut Sequence,
) -> Result<(), String> {
    {
        let guard = status.read().await;
        if guard.armed {
            return Ok(());
        }
        if guard.target.is_empty() {
            return Err("INVALID_OUTPUT_CONFIG: target belum dikonfigurasi".to_string());
        }
        validate_port_address(guard.port_address as u32, guard.compat_zero)?;
    }
    if destination.is_none() {
        return Err("INVALID_OUTPUT_CONFIG: target belum terselesaikan".to_string());
    }
    let bound = UdpSocket::bind("0.0.0.0:0")
        .map_err(|error| format!("INVALID_OUTPUT_CONFIG: UDP bind gagal: {error}"))?;
    *sequence = Sequence::new();
    *socket = Some(bound);
    let mut guard = status.write().await;
    guard.armed = true;
    guard.sequence = 1;
    guard.last_error = None;
    Ok(())
}

async fn stop_actor(
    status: &Arc<RwLock<OutputStatusCore>>,
    socket: &mut Option<UdpSocket>,
    destination: &Option<SocketAddr>,
    sequence: &mut Sequence,
    frame: &FrameBundle,
) -> Result<(), String> {
    let (armed, port_address) = {
        let guard = status.read().await;
        (guard.armed, guard.port_address)
    };
    if !armed {
        return Ok(());
    }
    let addr = destination
        .ok_or_else(|| "OUTPUT_STATE: destination tidak tersedia".to_string())?;
    let frames = build_stop_frames(&frame.dmx, port_address, sequence);

    {
        let mut guard = status.write().await;
        guard.armed = false;
    }

    {
        let sock = socket
            .as_ref()
            .ok_or_else(|| "OUTPUT_STATE: socket tidak tersedia".to_string())?;
        sock.send_to(&frames.first, addr)
            .map_err(|error| format!("OUTPUT_SEND_ERROR: {error}"))?;
    }
    tokio::time::sleep(OUTPUT_INTERVAL).await;
    {
        let sock = socket
            .as_ref()
            .ok_or_else(|| "OUTPUT_STATE: socket tidak tersedia".to_string())?;
        sock.send_to(&frames.second, addr)
            .map_err(|error| format!("OUTPUT_SEND_ERROR: {error}"))?;
    }
    *socket = None;

    let mut guard = status.write().await;
    guard.sequence = frames.sequence;
    guard.sent_frames = guard.sent_frames.saturating_add(2);
    Ok(())
}

async fn send_current_frame(
    frames: &Arc<RwLock<Arc<FrameBundle>>>,
    status: &Arc<RwLock<OutputStatusCore>>,
    socket: &UdpSocket,
    destination: SocketAddr,
    sequence: &mut Sequence,
) {
    let frame = frames.read().await.clone();
    let (port_address, compat_zero) = {
        let guard = status.read().await;
        (guard.port_address, guard.compat_zero)
    };
    if validate_port_address(port_address as u32, compat_zero).is_err() {
        let mut guard = status.write().await;
        guard.last_error = Some("INVALID_PORT_ADDRESS: alamat port tidak valid".to_string());
        return;
    }
    let packet = build_art_dmx(&frame.dmx, port_address, sequence.next());
    match socket.send_to(&packet, destination) {
        Ok(_) => {
            let mut guard = status.write().await;
            guard.sequence = packet[12];
            guard.sent_frames = guard.sent_frames.saturating_add(1);
            guard.last_error = None;
        }
        Err(error) => {
            eprintln!("[kinglight] error=udp_send detail={error}");
            let mut guard = status.write().await;
            guard.last_error = Some(format!("OUTPUT_SEND_ERROR: {error}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_dmx() -> [u8; DMX_LEN] {
        let mut dmx = [0u8; DMX_LEN];
        dmx[0] = 0xab;
        dmx[1] = 0xcd;
        dmx[511] = 0x7f;
        dmx
    }

    #[test]
    fn art_dmx_header_and_payload_match_wire_contract() {
        let dmx = sample_dmx();
        let packet = build_art_dmx(&dmx, 0x1234, 9);
        assert_eq!(packet.len(), 530);
        assert_eq!(&packet[0..8], b"Art-Net\0");
        assert_eq!(packet[8], 0x00);
        assert_eq!(packet[9], 0x50);
        assert_eq!(packet[10], 0x00);
        assert_eq!(packet[11], 0x0e);
        assert_eq!(packet[12], 9);
        assert_eq!(packet[13], 0);
        assert_eq!(packet[14], 0x34);
        assert_eq!(packet[15], 0x12);
        assert_eq!(packet[16], 0x02);
        assert_eq!(packet[17], 0x00);
        assert_eq!(&packet[18..530], &dmx[..]);
    }

    #[test]
    fn sequence_wraps_from_255_to_1_and_never_emits_zero() {
        let mut sequence = Sequence::new();
        let mut values = Vec::new();
        for _ in 0..260 {
            values.push(sequence.next());
        }
        assert_eq!(values[0], 1);
        assert_eq!(values[254], 255);
        assert_eq!(values[255], 1);
        assert!(values.iter().all(|value| *value >= 1));
    }

    #[test]
    fn port_address_is_encoded_as_raw_15_bits() {
        assert_eq!(encode_port_address(0), (0, 0));
        assert_eq!(encode_port_address(0x00ff), (0xff, 0));
        assert_eq!(encode_port_address(0x0100), (0x00, 0x01));
        assert_eq!(encode_port_address(MAX_PORT_ADDRESS), (0xff, 0x7f));
    }

    #[test]
    fn port_address_rejects_zero_unless_compat_zero_is_enabled() {
        assert!(validate_port_address(0, false).is_err());
        assert_eq!(validate_port_address(0, true), Ok(0));
        assert_eq!(validate_port_address(MAX_PORT_ADDRESS as u32, false), Ok(MAX_PORT_ADDRESS));
        assert!(validate_port_address(MAX_PORT_ADDRESS as u32 + 1, true).is_err());
    }

    #[test]
    fn stop_frames_share_payload_and_use_consecutive_sequences() {
        let dmx = sample_dmx();
        let mut sequence = Sequence::new();
        for _ in 0..254 {
            sequence.next();
        }
        let frames = build_stop_frames(&dmx, 7, &mut sequence);
        assert_eq!(frames.first[12], 255);
        assert_eq!(frames.second[12], 1);
        assert_eq!(&frames.first[18..], &frames.second[18..]);
        assert_eq!(frames.sequence, 1);
    }
}
