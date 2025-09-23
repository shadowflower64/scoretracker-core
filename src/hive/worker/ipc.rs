use crate::{debug, error, info, log_fn_name, log_should_print_debug};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::process;
use std::thread::{self, sleep};
use std::{io::Read, time::Duration};
use thiserror::Error;

const VERBOSE_CONNECTION_HANDLER: bool = true;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IncomingMessage {
    WhoAreYou,
}

impl IncomingMessage {
    pub fn receive(tcp_stream: &mut TcpStream) -> Result<Vec<u8>, Error> {
        log_fn_name!("incoming_message:receive");
        log_should_print_debug!(VERBOSE_CONNECTION_HANDLER);
        type E = Error;

        let mut size_bytes: [u8; 4] = [0, 0, 0, 0];
        tcp_stream
            .read_exact(&mut size_bytes)
            .map_err(E::IncomingMessageSizeNotRead)
            .unwrap();
        let size = u32::from_le_bytes(size_bytes) as usize;
        debug!("received size: {size} {size_bytes:?}");

        let mut message_bytes = vec![0u8; size];
        tcp_stream
            .read_exact(&mut message_bytes)
            .map_err(E::IncomingMessageContentNotRead)
            .unwrap();
        debug!("received message bytes: {message_bytes:?}");
        Ok(message_bytes)
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        log_fn_name!("incoming_message:parse");
        log_should_print_debug!(VERBOSE_CONNECTION_HANDLER);
        type E = Error;

        let message = serde_json::from_slice(bytes);
        debug!("parsed message: {message:?}");

        message.map_err(E::IncomingMessageNotDeserialized)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutgoingMessage {
    WhoAreYouResponse { name: String, pid: u32 },
}

impl OutgoingMessage {
    pub fn send(&self, tcp_stream: &mut TcpStream) -> Result<(), Error> {
        let json = serde_json::to_string(self).map_err(Error::OutgoingMessageNotSerialized)?;
        let bytes = json.as_bytes();
        let size = bytes.len() as u32;
        let size_bytes = size.to_le_bytes();
        tcp_stream.write_all(&size_bytes).map_err(Error::OutgoingMessageSizeNotSent)?;
        tcp_stream.write_all(bytes).map_err(Error::OutgoingMessageContentNotSent)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read size of the incoming message: {0}")]
    IncomingMessageSizeNotRead(io::Error),
    #[error("failed to read content of the incoming message: {0}")]
    IncomingMessageContentNotRead(io::Error),
    #[error("failed to deserialize incoming message to json: {0}")]
    IncomingMessageNotDeserialized(serde_json::Error),
    #[error("failed to serialize outgoing message to json: {0}")]
    OutgoingMessageNotSerialized(serde_json::Error),
    #[error("failed to send size of the outgoing message: {0}")]
    OutgoingMessageSizeNotSent(io::Error),
    #[error("failed to send content of the outgoing message: {0}")]
    OutgoingMessageContentNotSent(io::Error),
}

pub fn handle_connection(mut tcp_stream: TcpStream, peer_addr: SocketAddr) {
    thread::Builder::new()
        .name(format!("worker:conn:{}", peer_addr.port()))
        .spawn(move || {
            log_fn_name!("handler");
            info!("established connection with: {}", peer_addr);

            let message_bytes = IncomingMessage::receive(&mut tcp_stream).expect("failed to receive message");
            let message = IncomingMessage::parse(&message_bytes);
            match message {
                Ok(IncomingMessage::WhoAreYou) => {
                    let _ = OutgoingMessage::WhoAreYouResponse {
                        name: "test name".to_string(),
                        pid: process::id(),
                    }
                    .send(&mut tcp_stream)
                    .inspect_err(|e| error!("failed to send message: {e}; continuing"));
                }
                Err(e) => {
                    let message_bytes_as_string = String::from_utf8_lossy(&message_bytes);
                    error!("could not recognize received message: {e} - received message was: {message_bytes_as_string}");
                }
            }

            sleep(Duration::from_secs(5));

            tcp_stream.shutdown(Shutdown::Both).expect("failed to shutdown connection");
            info!("shutdown connection with: {}", peer_addr);
        })
        .expect("failed to create handler thread");
}

pub fn start_listener_thread(listener: TcpListener) {
    let address = listener.local_addr().expect("could not get local address of tcp listener");
    thread::Builder::new()
        .name("worker:tcp_listener".to_string())
        .spawn(move || {
            log_fn_name!("listener");
            info!("start listening on {address}");

            loop {
                match listener.accept() {
                    Ok((tcp_stream, peer_addr)) => {
                        handle_connection(tcp_stream, peer_addr);
                    }
                    Err(e) => {
                        error!("failed to establish connection with remote peer: {e}");
                    }
                }
            }
        })
        .expect("failed to create listener thread");
}
