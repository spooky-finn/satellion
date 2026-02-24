use std::{collections::HashMap, sync::Mutex};

use bip157::chain::IndexedHeader;
use diesel::result::Error;

use satellion_lib::{db::BlockHeader, db::CompactFilter, repository::ChainRepositoryTrait};

pub struct MockChainRepository {
    blocks: Mutex<HashMap<u32, BlockHeader>>,
    compact_filters: Mutex<HashMap<String, Vec<u8>>>,
}

impl MockChainRepository {
    pub fn new() -> Self {
        Self {
            blocks: Mutex::new(HashMap::new()),
            compact_filters: Mutex::new(HashMap::new()),
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
                blockhash: header.header.block_hash().to_string(),
                prev_blockhash: header.header.prev_blockhash.to_string(),
                time: header.header.time as i32,
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

    fn save_compact_filter(&self, blockhash: &str, filter_data: &[u8]) -> Result<(), Error> {
        let mut filters = self.compact_filters.lock().unwrap();
        filters.insert(blockhash.to_string(), filter_data.to_vec());
        Ok(())
    }

    fn get_compact_filter(
        &self,
        blockhash: &str,
    ) -> Result<Option<CompactFilter>, Error> {
        let filters = self.compact_filters.lock().unwrap();
        Ok(filters.get(blockhash).map(|data| CompactFilter {
            blockhash: blockhash.to_string(),
            filter_data: data.clone(),
        }))
    }
}
