use network::Node;
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

fn print_usage() {
    println!("Usage: basic_network <port> [peer_port]");
    println!("  <port>: Port number for this node");
    println!("  [peer_port]: Optional port number of the peer to connect to");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        print_usage();
        return Ok(());
    }

    let port: u16 = match args[1].parse() {
        Ok(p) => p,
        Err(_) => {
            println!("Invalid port number");
            print_usage();
            return Ok(());
        }
    };

    let peer_port: Option<u16> = if args.len() == 3 {
        match args[2].parse() {
            Ok(p) => Some(p),
            Err(_) => {
                println!("Invalid peer port number");
                print_usage();
                return Ok(());
            }
        }
    } else {
        None
    };

    let node_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let node = Node::new(node_addr).await?;

    if let Some(p_port) = peer_port {
        let peer_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), p_port);
        println!("Connecting to peer at port {}", p_port);
        node.connect_to_peer(peer_addr).await?;
    }

    println!("Starting node on port {}", port);
    node.start().await?;

    Ok(())
}
