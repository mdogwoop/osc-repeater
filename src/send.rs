use clap::Parser;
use rosc::{encoder, OscMessage, OscPacket, OscType};
use std::net::UdpSocket;

#[derive(Parser, Debug)]
#[command(name = "osc-send")]
#[command(about = "OSC message sender", long_about = None)]
struct Args {
    /// Target port
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let target = format!("127.0.0.1:{}", args.port);
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    let msg = OscMessage {
        addr: "/test".to_string(),
        args: vec![OscType::Int(101)],
    };

    let packet = OscPacket::Message(msg);
    let msg_buf = encoder::encode(&packet)?;

    socket.send_to(&msg_buf, &target)?;
    println!("Sent message to {}", target);

    Ok(())
}
