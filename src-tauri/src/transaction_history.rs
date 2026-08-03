use crate::{
    config::BlockChain,
    repository::{NewTx, TxRecord, TxRepository},
    utils,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletTxContext {
    pub wallet_name: String,
    pub chain: BlockChain,
    pub account_index: i32,
}

impl WalletTxContext {
    pub fn new(wallet_name: String, chain: BlockChain, account_index: i32) -> Self {
        Self {
            wallet_name,
            chain,
            account_index,
        }
    }
}

pub trait TransactionHistoryEntry {
    fn tx_hash(&self) -> &str;
    fn into_new_tx(self, context: WalletTxContext, created_at: i64) -> Result<NewTx, String>;
}

pub trait TransactionHistoryStore: Clone + Send + Sync + 'static {
    fn insert(&self, tx: NewTx) -> Result<TxRecord, String>;
    fn get(
        &self,
        wallet_name: &str,
        chain: BlockChain,
        tx_hash: &str,
    ) -> Result<Option<TxRecord>, String>;
}

impl TransactionHistoryStore for TxRepository {
    fn insert(&self, tx: NewTx) -> Result<TxRecord, String> {
        TxRepository::insert(self, tx)
    }

    fn get(
        &self,
        wallet_name: &str,
        chain: BlockChain,
        tx_hash: &str,
    ) -> Result<Option<TxRecord>, String> {
        TxRepository::get(self, wallet_name, chain, tx_hash)
    }
}

#[derive(Clone, Debug)]
pub struct TransactionHistoryService<S = TxRepository> {
    store: S,
}

impl<S> TransactionHistoryService<S>
where
    S: TransactionHistoryStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn record<E>(&self, context: WalletTxContext, entry: E) -> Result<TxRecord, String>
    where
        E: TransactionHistoryEntry,
    {
        let tx_hash = entry.tx_hash().to_string();
        let tx = entry.into_new_tx(context.clone(), utils::now() as i64)?;

        match self.store.insert(tx) {
            Ok(record) => Ok(record),
            Err(err) if err.starts_with("tx already recorded:") => self
                .store
                .get(&context.wallet_name, context.chain, &tx_hash)?
                .ok_or(err),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use serde_json::Value;

    use super::*;
    use crate::{
        config::BlockChain,
        repository::{TxDirection, TxStatus},
    };

    #[derive(Clone, Default)]
    struct MemoryStore {
        txs: Arc<Mutex<Vec<NewTx>>>,
    }

    impl TransactionHistoryStore for MemoryStore {
        fn insert(&self, tx: NewTx) -> Result<TxRecord, String> {
            let mut txs = self.txs.lock().unwrap();
            if txs.iter().any(|existing| {
                existing.wallet_name == tx.wallet_name
                    && existing.chain == tx.chain
                    && existing.tx_hash == tx.tx_hash
            }) {
                return Err(format!("tx already recorded: {}", tx.tx_hash));
            }
            txs.push(tx.clone());
            Ok(record_from(tx))
        }

        fn get(
            &self,
            wallet_name: &str,
            chain: BlockChain,
            tx_hash: &str,
        ) -> Result<Option<TxRecord>, String> {
            Ok(self
                .txs
                .lock()
                .unwrap()
                .iter()
                .find(|tx| {
                    tx.wallet_name == wallet_name && tx.chain == chain && tx.tx_hash == tx_hash
                })
                .cloned()
                .map(record_from))
        }
    }

    struct TestEntry {
        tx_hash: String,
    }

    impl TransactionHistoryEntry for TestEntry {
        fn tx_hash(&self) -> &str {
            &self.tx_hash
        }

        fn into_new_tx(self, context: WalletTxContext, created_at: i64) -> Result<NewTx, String> {
            Ok(NewTx {
                wallet_name: context.wallet_name,
                chain: context.chain,
                account_index: context.account_index,
                tx_hash: self.tx_hash,
                direction: TxDirection::Outgoing,
                status: TxStatus::Pending,
                from_address: None,
                to_address: None,
                amount: "1".into(),
                fee: None,
                block_height: None,
                chain_data: Value::Object(Default::default()),
                created_at,
            })
        }
    }

    fn record_from(tx: NewTx) -> TxRecord {
        TxRecord {
            tx_hash: tx.tx_hash,
            wallet_name: tx.wallet_name,
            chain: tx.chain,
            account_index: tx.account_index,
            direction: tx.direction,
            status: tx.status,
            from_address: tx.from_address,
            to_address: tx.to_address,
            amount: tx.amount,
            fee: tx.fee,
            block_height: tx.block_height.map(|h| h.to_string()),
            chain_data: tx.chain_data.to_string(),
            created_at: tx.created_at.to_string(),
            confirmed_at: None,
        }
    }

    #[test]
    fn duplicate_record_returns_existing_row() {
        let service = TransactionHistoryService::new(MemoryStore::default());
        let context = WalletTxContext::new("alice".into(), BlockChain::Ethereum, 0);

        let first = service
            .record(
                context.clone(),
                TestEntry {
                    tx_hash: "0xabc".into(),
                },
            )
            .unwrap();
        let second = service
            .record(
                context,
                TestEntry {
                    tx_hash: "0xabc".into(),
                },
            )
            .unwrap();

        assert_eq!(first.tx_hash, second.tx_hash);
    }
}
