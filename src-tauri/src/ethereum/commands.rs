use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_provider::{Provider, RootProvider};

#[derive(serde::Serialize)]
pub struct ChainInfo {
    block_number: u64,
    block_hash: String,
    base_fee_per_gas: Option<u64>,
}

#[tauri::command]
pub async fn eth_chain_info(client: tauri::State<'_, RootProvider>) -> Result<ChainInfo, String> {
    let block = client
        .get_block(BlockId::Number(BlockNumberOrTag::Latest))
        .await
        .map_err(|e| e.to_string())?;

    // let block = block.inspect(f | println!("Block: {:?}", f));

    // block.iter().for_each(|b| println!("Block: {:?}", b));
    if !block.is_some() {
        return Err("Block not found".to_string());
    }
    let block = block.unwrap();
    Ok(ChainInfo {
        block_number: block.header.number,
        block_hash: block.header.hash.to_string(),
        base_fee_per_gas: block.header.base_fee_per_gas,
    })
}
