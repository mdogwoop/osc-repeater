use clap::Parser;
use rosc::{encoder, OscPacket};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

#[derive(Parser, Debug)]
#[command(name = "osc-repeater")]
#[command(about = "OSC message repeater", long_about = None)]
struct Args {
    /// Debug logging
    #[arg(short, long)]
    debug: bool,

    /// Configuration file
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    #[serde(rename = "listenPorts")]
    listen_ports: Vec<u16>,
    targets: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Set up logging
    if args.debug {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    debug!("Debug mode enabled");

    // Load configuration
    let config_file = std::fs::File::open(&args.config)?;
    let config: Config = serde_yaml::from_reader(config_file)?;

    info!("Loaded configuration: {:?}", config);

    // Create distributor
    let (tx, rx) = mpsc::unbounded_channel();
    let distributor = Distributor::new(config.targets).await?;
    
    // Spawn distributor task
    let _distributor_handle = tokio::spawn(async move {
        distributor.run(rx).await;
    });

    // Create receivers for each listen port
    let mut receiver_handles = Vec::new();
    for port in config.listen_ports {
        let tx = tx.clone();
        let handle = tokio::spawn(async move {
            if let Err(e) = run_receiver(port, tx).await {
                error!("Receiver on port {} failed: {}", port, e);
            }
        });
        receiver_handles.push(handle);
    }

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}

async fn run_receiver(
    port: u16,
    tx: mpsc::UnboundedSender<Arc<OscPacket>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("0.0.0.0:{}", port);
    let socket = UdpSocket::bind(&addr).await?;
    info!("Receiver listening on {}", addr);

    let mut buf = vec![0u8; rosc::decoder::MTU];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, _src)) => {
                match rosc::decoder::decode_udp(&buf[..size]) {
                    Ok((_, packet)) => {
                        debug!("Received message: {:?}", packet);
                        if let Err(e) = tx.send(Arc::new(packet)) {
                            error!("Failed to send to distributor: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to decode OSC packet: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to receive from socket: {}", e);
            }
        }
    }
}

struct Distributor {
    senders: Vec<Arc<Sender>>,
}

impl Distributor {
    async fn new(targets: Vec<String>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut senders = Vec::new();
        for target in targets {
            let sender = Arc::new(Sender::new(&target).await?);
            senders.push(sender);
        }
        Ok(Self { senders })
    }

    async fn run(self, mut rx: mpsc::UnboundedReceiver<Arc<OscPacket>>) {
        while let Some(packet) = rx.recv().await {
            debug!("Distributing message: {:?}", packet);
            for sender in &self.senders {
                sender.send(Arc::clone(&packet)).await;
            }
        }
    }
}

struct Sender {
    socket: UdpSocket,
    target: SocketAddr,
}

impl Sender {
    async fn new(target: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let target_addr: SocketAddr = target.parse()?;
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        info!("Starting new Sender to {}", target_addr);
        Ok(Self {
            socket,
            target: target_addr,
        })
    }

    async fn send(&self, packet: Arc<OscPacket>) {
        match encoder::encode(&*packet) {
            Ok(msg_buf) => {
                debug!("Sending message to {}: {:?}", self.target, packet);
                if let Err(e) = self.socket.send_to(&msg_buf, self.target).await {
                    error!("Failed to send to {}: {}", self.target, e);
                }
            }
            Err(e) => {
                error!("Failed to encode OSC packet: {}", e);
            }
        }
    }
}
