
use std::{net::SocketAddrV4, str::FromStr};

use bip157::{ Builder, TrustedPeer};

pub async fn init_neutrino() {
    let peer= TrustedPeer::from_socket_addr(SocketAddrV4::from_str("127.0.0.1:18444").unwrap());

    let (node, _) = Builder::new(bip157::Network::Regtest)
    .required_peers(1)
    .add_peers(vec![peer])
    .build();

    let result = node.run().await;
    match result {
        Ok(_) => println!("Node started"),
        Err(e) => println!("Error starting node: {}", e),
    }
}