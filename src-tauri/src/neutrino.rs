use crate::app_state::AppState;
use crate::db::BlockHeader;
use crate::schema;
use bip157::chain::{BlockHeaderChanges, IndexedHeader};
use bip157::{Builder, Event, TrustedPeer};
use diesel::SqliteConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use std::net::SocketAddrV4;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

const REGTEST_PEER: &str = "127.0.0.1:18444";

pub struct Neutrino {
    pub node: bip157::Node,
    pub client: bip157::Client,
}

impl Neutrino {
    pub fn connect_regtest() -> Result<Self, String> {
        let socket_addr = match SocketAddrV4::from_str(REGTEST_PEER) {
            Ok(addr) => addr,
            Err(e) => {
                return Err(format!("Error parsing socket address: {e:?}"));
            }
        };
        let peer = TrustedPeer::from_socket_addr(socket_addr);
        let (node, client) = Builder::new(bip157::Network::Regtest)
            .required_peers(1)
            .add_peers(vec![peer])
            .response_timeout(Duration::from_secs(10))
            .build();
        Ok(Self { node, client })
    }
}

/// Handle incoming neutrino events and update shared state
pub async fn handle_chain_updates(
    mut client: bip157::Client,
    app_state: Arc<AppState>,
    db_pool: Pool<ConnectionManager<SqliteConnection>>,
) {
    let block_height = app_state.chain_height.clone();
    let sync_completed = app_state.sync_completed.clone();
    let mut conn = db_pool.get().expect("Error getting connection from pool");

    while let Some(event) = client.event_rx.recv().await {
        match event {
            Event::FiltersSynced(sync_update) => {
                *block_height.lock().unwrap() = sync_update.tip.height;
                *sync_completed.lock().unwrap() = true;
                println!("Synced to height: {}", sync_update.tip.height);
            }
            Event::ChainUpdate(changes) => {
                let new_height = match changes {
                    BlockHeaderChanges::Connected(header) => {
                        if let Err(e) = save_block_header(&mut conn, header) {
                            eprintln!("Error inserting block: {e:?}");
                        }
                        Some(header.height)
                    }
                    BlockHeaderChanges::Reorganized { accepted, .. } => {
                        accepted.last().map(|h| h.height)
                    }
                    BlockHeaderChanges::ForkAdded(_) => None,
                };

                match new_height {
                    Some(h) => {
                        *block_height.lock().unwrap() = h;
                        *sync_completed.lock().unwrap() = false;
                    }
                    None => {
                        eprintln!("Chain error: no new height");
                        *sync_completed.lock().unwrap() = true;
                    }
                }
            }
            Event::Block(_block) => {}
            Event::IndexedFilter(_filter) => {}
        }
    }
}

fn save_block_header(
    conn: &mut SqliteConnection,
    block_header: IndexedHeader,
) -> Result<usize, diesel::result::Error> {
    diesel::insert_into(schema::block_headers::table)
        .values(&BlockHeader {
            height: block_header.height as i32,
            merkle_root: block_header.header.merkle_root.to_string(),
            prev_blockhash: block_header.header.prev_blockhash.to_string(),
            time: block_header.header.time as i32,
            version: block_header.header.version.to_consensus(),
            bits: block_header.header.bits.to_consensus() as i32,
            nonce: block_header.header.nonce as i32,
        })
        .execute(conn)
}
