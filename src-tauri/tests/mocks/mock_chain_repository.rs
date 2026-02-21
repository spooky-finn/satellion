use std::{collections::HashMap, sync::Mutex};

use bip157::chain::IndexedHeader;
use diesel::result::Error;

use satellion_lib::{db::BlockHeader, repository::ChainRepositoryTrait};

pub struct MockChainRepository {
    blocks: Mutex<HashMap<u32, BlockHeader>>,
}

impl MockChainRepository {
    pub fn new() -> Self {
        Self {
            blocks: Mutex::new(HashMap::new()),
        }
    }
}

impl ChainRepositoryTrait for MockChainRepository {
    fn save_block_header(&self, header: &IndexedHeader) -> Result<usize, Error> {
        let mut blocks = self.blocks.lock().unwrap();

        let height = header.height;
        if blocks.contains_key(&height) {
            return Ok(0);
        }

        blocks.insert(
            height,
            BlockHeader {
                height: height as i32,
                merkle_root: header.header.merkle_root.to_string(),
                prev_blockhash: header.header.prev_blockhash.to_string(),
                time: header.header.time as i32,
                version: header.header.version.to_consensus(),
                bits: header.header.bits.to_consensus() as i32,
                nonce: header.header.nonce as i32,
            },
        );

        Ok(1)
    }

    fn last_block(&self) -> Result<BlockHeader, Error> {
        let blocks = self.blocks.lock().unwrap();
        blocks
            .values()
            .max_by_key(|b| b.height)
            .cloned()
            .ok_or(Error::NotFound)
    }

    fn get_block_header(&self, height: u32) -> Result<Option<BlockHeader>, Error> {
        let blocks = self.blocks.lock().unwrap();
        Ok(blocks.get(&height).cloned())
    }
}
