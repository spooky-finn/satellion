use std::collections::HashMap;

use bitcoin::{Network, OutPoint, Transaction, consensus, hex::FromHex};
use chrono::{TimeZone, Utc};
use diesel::{prelude::*, r2d2::ConnectionManager, result::Error};
use r2d2::Pool;
use serde::{Deserialize, Serialize};
use specta::Type;
use strum::FromRepr;
use strum::{AsRefStr, Display, EnumString};

use crate::{
    config::BlockChain,
    repository::base_repository::BaseRepository,
    schema::{btc_broadcast_inputs as inputs, btc_broadcast_payloads as payloads, transactions},
};

#[repr(i16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, FromRepr)]
#[serde(rename_all = "lowercase")]
pub enum TxDirection {
    Incoming = 0,
    Outgoing = 1,
    SelfTransfer = 2,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, AsRefStr, Display, EnumString,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum TxStatus {
    Unknown,
    Pending,
    Confirmed,
    Failed,
}

fn chain_as_str(chain: BlockChain) -> &'static str {
    match chain {
        BlockChain::Bitcoin => "bitcoin",
        BlockChain::Ethereum => "ethereum",
    }
}

fn chain_from_str(s: &str) -> Result<BlockChain, String> {
    match s {
        "bitcoin" => Ok(BlockChain::Bitcoin),
        "ethereum" => Ok(BlockChain::Ethereum),
        other => Err(format!("unknown chain: {other}")),
    }
}

fn unix_timestamp_to_iso(ts: i64) -> Result<String, String> {
    Utc.timestamp_opt(ts, 0)
        .single()
        .map(|dt| dt.to_rfc3339())
        .ok_or_else(|| format!("invalid unix timestamp: {ts}"))
}

/// Inputs for inserting a new transaction record. `chain_data` is the
/// chain-specific JSON payload — see e.g. [`BtcChainData`] / [`EthChainData`].
#[derive(Debug, Clone)]
pub struct NewTx {
    pub wallet_name: String,
    pub chain: BlockChain,
    pub account_index: i32,
    pub tx_hash: String,
    pub direction: TxDirection,
    pub status: TxStatus,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub amount: i64,
    pub fee: Option<i32>,
    pub block_height: Option<i64>,
    pub chain_data: serde_json::Value,
    pub created_at: i64,
}

#[derive(Insertable)]
#[diesel(table_name = transactions)]
struct NewTxRow<'a> {
    wallet_name: &'a str,
    chain: &'a str,
    account_index: i32,
    tx_hash: &'a str,
    direction: i16,
    status: &'a str,
    from_address: Option<&'a str>,
    to_address: Option<&'a str>,
    amount: i64,
    fee: Option<i32>,
    block_height: Option<i64>,
    chain_data: String,
    created_at: i64,
}

impl<'a> TryFrom<&'a NewTx> for NewTxRow<'a> {
    type Error = String;

    fn try_from(tx: &'a NewTx) -> Result<Self, String> {
        let chain_data_str =
            serde_json::to_string(&tx.chain_data).map_err(|e| format!("encode chain_data: {e}"))?;
        Ok(Self {
            wallet_name: tx.wallet_name.as_str(),
            chain: chain_as_str(tx.chain),
            account_index: tx.account_index,
            tx_hash: tx.tx_hash.as_str(),
            direction: tx.direction as i16,
            status: tx.status.as_ref(),
            from_address: tx.from_address.as_deref(),
            to_address: tx.to_address.as_deref(),
            amount: tx.amount,
            fee: tx.fee,
            block_height: tx.block_height,
            chain_data: chain_data_str,
            created_at: tx.created_at,
        })
    }
}

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = transactions)]
struct TxRow {
    tx_hash: String,
    wallet_name: String,
    chain: String,
    account_index: i32,
    direction: i16,
    status: String,
    from_address: Option<String>,
    to_address: Option<String>,
    amount: i64,
    fee: Option<i32>,
    block_height: Option<i64>,
    chain_data: String,
    created_at: i64,
    confirmed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct TxRecord {
    pub tx_hash: String,
    pub wallet_name: String,
    pub chain: BlockChain,
    pub account_index: i32,
    pub direction: TxDirection,
    pub status: TxStatus,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub amount: String,
    pub fee: Option<i32>,
    pub block_height: Option<String>,
    /// Chain-specific JSON payload. Stored verbatim so the frontend can decode
    /// it according to the [`chain`] discriminator.
    pub chain_data: String,
    pub created_at: String,
    pub confirmed_at: Option<String>,
}

impl TryFrom<TxRow> for TxRecord {
    type Error = String;

    fn try_from(row: TxRow) -> Result<Self, Self::Error> {
        Ok(Self {
            tx_hash: row.tx_hash,
            wallet_name: row.wallet_name,
            chain: chain_from_str(&row.chain)?,
            account_index: row.account_index,
            direction: TxDirection::from_repr(row.direction)
                .ok_or_else(|| format!("unknown tx direction: {}", row.direction))?,
            status: row
                .status
                .parse()
                .map_err(|e| format!("unknown tx status '{}': {e}", row.status))?,
            from_address: row.from_address,
            to_address: row.to_address,
            amount: row.amount.to_string(),
            fee: row.fee,
            block_height: row.block_height.map(|h| h.to_string()),
            chain_data: row.chain_data,
            created_at: unix_timestamp_to_iso(row.created_at)?,
            confirmed_at: row.confirmed_at.map(unix_timestamp_to_iso).transpose()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TxQuery {
    pub wallet_name: String,
    pub chain: BlockChain,
    pub account_index: i32,
    pub limit: Option<i64>,
}

struct RecordBtcError(String);

impl From<diesel::result::Error> for RecordBtcError {
    fn from(error: diesel::result::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<String> for RecordBtcError {
    fn from(error: String) -> Self {
        Self(error)
    }
}

pub struct BtcBroadcastScope<'a> {
    pub wallet_name: &'a str,
    pub account_index: i32,
    pub network: Network,
    pub wallet_key: &'a str,
}

impl BtcBroadcastScope<'_> {
    fn payloads(&self) -> payloads::BoxedQuery<'_, diesel::sqlite::Sqlite> {
        payloads::table
            .filter(payloads::wallet_name.eq(self.wallet_name))
            .filter(payloads::account_index.eq(self.account_index))
            .filter(payloads::network.eq(self.network.to_string()))
            .filter(payloads::wallet_key.eq(self.wallet_key))
            .into_boxed()
    }

    fn unknown_attempts(&self) -> transactions::BoxedQuery<'_, diesel::sqlite::Sqlite> {
        transactions::table
            .filter(transactions::wallet_name.eq(self.wallet_name))
            .filter(transactions::chain.eq(chain_as_str(BlockChain::Bitcoin)))
            .filter(transactions::account_index.eq(self.account_index))
            .filter(transactions::status.eq(TxStatus::Unknown.as_ref().to_owned()))
            .filter(transactions::tx_hash.eq_any(self.payloads().select(payloads::txid)))
            .into_boxed()
    }
}

#[derive(Clone, Debug)]
pub struct TxRepository {
    base: BaseRepository,
}

impl TxRepository {
    pub fn new(db_pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self {
            base: BaseRepository::new(db_pool),
        }
    }

    pub fn reserved_btc_inputs(
        &self,
        network: Network,
    ) -> Result<HashMap<OutPoint, String>, String> {
        let mut conn = self.base.get_conn().map_err(|e| e.to_string())?;
        inputs::table
            .filter(inputs::network.eq(network.to_string()))
            .select((inputs::outpoint, inputs::txid))
            .load::<(String, String)>(&mut conn)
            .map_err(|e| format!("load reserved Bitcoin inputs: {e}"))?
            .into_iter()
            .map(|(outpoint, txid)| {
                outpoint
                    .parse()
                    .map(|outpoint| (outpoint, txid))
                    .map_err(|e| format!("invalid reserved Bitcoin input: {e}"))
            })
            .collect()
    }

    /// Save the transaction and reserve its inputs in the same SQLite transaction.
    pub fn record_btc_attempt(
        &self,
        scope: &BtcBroadcastScope<'_>,
        signed: &Transaction,
        tx: NewTx,
    ) -> Result<(), String> {
        if !matches!(tx.chain, BlockChain::Bitcoin)
            || tx.status != TxStatus::Unknown
            || tx.wallet_name != scope.wallet_name
            || tx.account_index != scope.account_index
            || tx.tx_hash != signed.compute_txid().to_string()
        {
            return Err("Bitcoin transaction record does not match the broadcast attempt".into());
        }
        let row = NewTxRow::try_from(&tx)?;
        let network = scope.network.to_string();
        let mut conn = self.base.get_conn().map_err(|e| e.to_string())?;
        conn.immediate_transaction::<_, RecordBtcError, _>(|conn| {
            if let Some(recipient) = &tx.to_address {
                let unresolved = scope.unknown_attempts()
                    .filter(transactions::to_address.eq(recipient))
                    .filter(transactions::amount.eq(tx.amount))
                    .select(transactions::tx_hash)
                    .first::<String>(conn)
                    .optional()
                    ?;
                if let Some(txid) = unresolved {
                    return Err(format!("A payment of this amount to this recipient is unresolved ({txid}). Use Retry same transaction to resend it.").into());
                }
            }

            for input in &signed.input {
                let outpoint = input.previous_output.to_string();
                let inserted = diesel::insert_into(inputs::table)
                    .values((
                        inputs::network.eq(&network),
                        inputs::outpoint.eq(&outpoint),
                        inputs::txid.eq(&tx.tx_hash),
                        inputs::created_at.eq(tx.created_at),
                    ))
                    .on_conflict((inputs::network, inputs::outpoint))
                    .do_nothing()
                    .execute(conn)
                    ?;
                if inserted == 0 {
                    let txid = inputs::table.find((&network, &outpoint))
                        .select(inputs::txid)
                        .first::<String>(conn)
                        ?;
                    return Err(format!("Input {outpoint} is already reserved by transaction {txid}.").into());
                }
            }
            diesel::insert_into(payloads::table)
                .values((
                    payloads::txid.eq(&tx.tx_hash),
                    payloads::wallet_name.eq(scope.wallet_name),
                    payloads::account_index.eq(scope.account_index),
                    payloads::network.eq(&network),
                    payloads::raw_transaction.eq(consensus::encode::serialize_hex(signed)),
                    payloads::wallet_key.eq(scope.wallet_key),
                ))
                .execute(conn)
                ?;
            diesel::insert_into(transactions::table)
                .values(&row)
                .execute(conn)
                ?;
            Ok(())
        }).map_err(|e| e.0)
    }

    pub fn btc_attempt_payload(
        &self,
        scope: &BtcBroadcastScope<'_>,
        txid: &str,
    ) -> Result<Transaction, String> {
        let mut conn = self.base.get_conn().map_err(|e| e.to_string())?;
        let raw = scope
            .payloads()
            .filter(payloads::txid.eq(txid))
            .filter(payloads::txid.eq_any(scope.unknown_attempts().select(transactions::tx_hash)))
            .select(payloads::raw_transaction)
            .first::<String>(&mut conn)
            .map_err(|e| format!("Saved unresolved transaction is unavailable: {e}"))?;
        let bytes =
            Vec::<u8>::from_hex(&raw).map_err(|e| format!("Invalid saved transaction: {e}"))?;
        let transaction: Transaction = consensus::deserialize(&bytes)
            .map_err(|e| format!("Invalid saved transaction: {e}"))?;
        if transaction.compute_txid().to_string() != txid {
            return Err("Saved transaction ID mismatch".into());
        }
        Ok(transaction)
    }

    pub fn unresolved_btc_attempts(
        &self,
        scope: &BtcBroadcastScope<'_>,
    ) -> Result<Vec<TxRecord>, String> {
        let mut conn = self.base.get_conn().map_err(|e| e.to_string())?;
        scope
            .unknown_attempts()
            .order(transactions::created_at.desc())
            .select(TxRow::as_select())
            .load::<TxRow>(&mut conn)
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(TxRecord::try_from)
            .collect()
    }

    pub fn acknowledge_btc_attempt(
        &self,
        scope: &BtcBroadcastScope<'_>,
        txid: &str,
    ) -> Result<(), String> {
        let mut conn = self.base.get_conn().map_err(|e| e.to_string())?;
        diesel::update(
            transactions::table
                .filter(transactions::wallet_name.eq(scope.wallet_name))
                .filter(transactions::chain.eq(chain_as_str(BlockChain::Bitcoin)))
                .filter(transactions::account_index.eq(scope.account_index))
                .filter(transactions::status.eq(TxStatus::Unknown.as_ref()))
                .filter(transactions::tx_hash.eq(txid))
                .filter(transactions::tx_hash.eq_any(scope.payloads().select(payloads::txid))),
        )
        .set(transactions::status.eq(TxStatus::Pending.as_ref()))
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn insert(&self, tx: NewTx) -> Result<TxRecord, String> {
        let row = NewTxRow::try_from(&tx)?;
        let mut conn = self.base.get_conn().map_err(|e| e.to_string())?;
        let inserted: TxRow = diesel::insert_into(transactions::table)
            .values(&row)
            .returning(TxRow::as_returning())
            .get_result(&mut conn)
            .map_err(|e| match e {
                Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _) => {
                    format!("tx already recorded: {}", tx.tx_hash)
                }
                other => other.to_string(),
            })?;
        TxRecord::try_from(inserted)
    }

    pub fn list(&self, q: &TxQuery) -> Result<Vec<TxRecord>, String> {
        use crate::schema::transactions::dsl::*;
        let mut conn = self.base.get_conn().map_err(|e| e.to_string())?;

        let rows: Vec<TxRow> = transactions
            .into_boxed()
            .filter(wallet_name.eq(&q.wallet_name))
            .filter(chain.eq(chain_as_str(q.chain)))
            .filter(account_index.eq(q.account_index))
            .order(created_at.desc())
            .limit(q.limit.unwrap_or(200))
            .select(TxRow::as_select())
            .load(&mut conn)
            .map_err(|e| e.to_string())?;

        rows.into_iter().map(TxRecord::try_from).collect()
    }

    pub fn update_status(
        &self,
        wallet: &str,
        chain_id: BlockChain,
        hash: &str,
        new_status: TxStatus,
        confirmed_block: Option<i64>,
        confirmed_at_ts: Option<i64>,
    ) -> Result<usize, String> {
        use crate::schema::transactions::dsl::*;

        let mut conn = self.base.get_conn().map_err(|e| e.to_string())?;
        diesel::update(
            transactions
                .filter(wallet_name.eq(wallet))
                .filter(chain.eq(chain_as_str(chain_id)))
                .filter(tx_hash.eq(hash)),
        )
        .set((
            status.eq(new_status.as_ref()),
            block_height.eq(confirmed_block),
            confirmed_at.eq(confirmed_at_ts),
        ))
        .execute(&mut conn)
        .map_err(|e| e.to_string())
    }

    pub fn rename_btc_broadcast_records(
        &self,
        old_name: &str,
        new_name: &str,
        wallet_key: &str,
    ) -> Result<(), String> {
        let mut conn = self.base.get_conn().map_err(|e| e.to_string())?;
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let owned_payloads = payloads::table
                .filter(payloads::wallet_name.eq(old_name))
                .filter(payloads::wallet_key.eq(wallet_key));
            diesel::update(
                transactions::table
                    .filter(transactions::wallet_name.eq(old_name))
                    .filter(transactions::chain.eq(chain_as_str(BlockChain::Bitcoin)))
                    .filter(diesel::dsl::exists(
                        owned_payloads
                            .filter(payloads::account_index.eq(transactions::account_index))
                            .filter(payloads::txid.eq(transactions::tx_hash)),
                    )),
            )
            .set(transactions::wallet_name.eq(new_name))
            .execute(conn)?;
            diesel::update(owned_payloads)
                .set(payloads::wallet_name.eq(new_name))
                .execute(conn)?;
            Ok(())
        })
        .map_err(|e| format!("Could not move Bitcoin broadcast history: {e}"))
    }

    pub fn delete_for_wallet(&self, wallet: &str) -> Result<usize, String> {
        use crate::schema::transactions::dsl::*;

        let mut conn = self.base.get_conn().map_err(|e| e.to_string())?;
        diesel::delete(transactions.filter(wallet_name.eq(wallet)))
            .execute(&mut conn)
            .map_err(|e| e.to_string())
    }
}

/// Chain-specific JSON payload stored in `transactions.chain_data` for Bitcoin
/// transactions.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BtcChainData {
    pub vsize: Option<u32>,
    pub rbf: bool,
    /// Set on a CPFP child transaction to point at the parent it bumps.
    pub parent_tx_id: Option<String>,
    pub change_value_sat: Option<u64>,
}

/// Chain-specific JSON payload stored in `transactions.chain_data` for Ethereum
/// transactions.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EthChainData {
    pub nonce: u64,
    pub token_address: String,
    pub token_symbol: String,
    pub token_decimals: u8,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub gas_limit: String,
}

#[cfg(test)]
mod tests {
    use diesel_migrations::MigrationHarness;

    use super::*;
    use crate::db::MIGRATIONS;

    fn make_pool() -> Pool<ConnectionManager<SqliteConnection>> {
        let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool = Pool::builder().max_size(1).build(manager).unwrap();
        let mut conn = pool.get().unwrap();
        conn.run_pending_migrations(MIGRATIONS).unwrap();
        pool
    }

    #[test]
    fn attempt_history_and_inputs_commit_or_roll_back_together() {
        let repo = TxRepository::new(make_pool());
        let input = bitcoin::TxIn::default();
        let mut tx = bitcoin::Transaction {
            version: bitcoin::transaction::Version::TWO,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![input.clone()],
            output: vec![],
        };
        let record = |tx: &bitcoin::Transaction| NewTx {
            wallet_name: "wallet".into(),
            chain: BlockChain::Bitcoin,
            account_index: 0,
            tx_hash: tx.compute_txid().to_string(),
            direction: TxDirection::Outgoing,
            status: TxStatus::Unknown,
            from_address: None,
            to_address: None,
            amount: 100,
            fee: Some(5),
            block_height: None,
            chain_data: serde_json::json!({}),
            created_at: 1,
        };
        let scope = BtcBroadcastScope {
            wallet_name: "wallet",
            account_index: 0,
            network: Network::Bitcoin,
            wallet_key: "key",
        };
        repo.record_btc_attempt(&scope, &tx, record(&tx)).unwrap();
        let txid = tx.compute_txid().to_string();
        assert_eq!(repo.unresolved_btc_attempts(&scope).unwrap().len(), 1);
        assert_eq!(repo.btc_attempt_payload(&scope, &txid).unwrap(), tx);
        for other_scope in [
            BtcBroadcastScope {
                wallet_name: "other",
                ..scope
            },
            BtcBroadcastScope {
                wallet_key: "other",
                ..scope
            },
            BtcBroadcastScope {
                account_index: 1,
                ..scope
            },
            BtcBroadcastScope {
                network: Network::Regtest,
                ..scope
            },
        ] {
            assert!(
                repo.unresolved_btc_attempts(&other_scope)
                    .unwrap()
                    .is_empty()
            );
            assert!(repo.btc_attempt_payload(&other_scope, &txid).is_err());
        }
        let mut new_input = input.clone();
        new_input.previous_output.vout = 1;
        tx.input = vec![new_input.clone(), input];
        assert!(repo.record_btc_attempt(&scope, &tx, record(&tx)).is_err());
        let reserved = repo.reserved_btc_inputs(bitcoin::Network::Bitcoin).unwrap();
        assert_eq!(reserved.len(), 1);
        assert!(
            repo.reserved_btc_inputs(bitcoin::Network::Regtest)
                .unwrap()
                .is_empty()
        );
        assert!(!reserved.contains_key(&new_input.previous_output));
        let rows = repo
            .list(&TxQuery {
                wallet_name: "wallet".into(),
                chain: BlockChain::Bitcoin,
                account_index: 0,
                limit: None,
            })
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, TxStatus::Unknown);
        repo.rename_btc_broadcast_records("wallet", "renamed", "other")
            .unwrap();
        assert!(repo.btc_attempt_payload(&scope, &txid).is_ok());
        repo.rename_btc_broadcast_records("wallet", "renamed", scope.wallet_key)
            .unwrap();
        assert!(repo.btc_attempt_payload(&scope, &txid).is_err());
        let renamed = BtcBroadcastScope {
            wallet_name: "renamed",
            ..scope
        };
        assert!(repo.btc_attempt_payload(&renamed, &txid).is_ok());
        repo.update_status(
            "renamed",
            BlockChain::Bitcoin,
            &txid,
            TxStatus::Confirmed,
            Some(42),
            None,
        )
        .unwrap();
        repo.acknowledge_btc_attempt(&renamed, &txid).unwrap();
        assert!(repo.btc_attempt_payload(&renamed, &txid).is_err());
        let records = repo
            .list(&TxQuery {
                wallet_name: "renamed".into(),
                chain: BlockChain::Bitcoin,
                account_index: 0,
                limit: None,
            })
            .unwrap();
        assert_eq!(records[0].status, TxStatus::Confirmed);
        assert_eq!(records[0].block_height.as_deref(), Some("42"));
    }

    #[test]
    fn unresolved_payment_blocks_same_recipient_and_amount_with_different_inputs() {
        let repo = TxRepository::new(make_pool());
        let scope = BtcBroadcastScope {
            wallet_name: "wallet",
            account_index: 0,
            network: Network::Bitcoin,
            wallet_key: "key",
        };
        let mut tx = Transaction {
            version: bitcoin::transaction::Version::TWO,
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![bitcoin::TxIn::default()],
            output: vec![],
        };
        let record = |tx: &Transaction| NewTx {
            wallet_name: scope.wallet_name.into(),
            chain: BlockChain::Bitcoin,
            account_index: scope.account_index,
            tx_hash: tx.compute_txid().to_string(),
            direction: TxDirection::Outgoing,
            status: TxStatus::Unknown,
            from_address: None,
            to_address: Some("recipient".into()),
            amount: 100,
            fee: Some(5),
            block_height: None,
            chain_data: serde_json::json!({}),
            created_at: 1,
        };
        repo.record_btc_attempt(&scope, &tx, record(&tx)).unwrap();
        let first_id = tx.compute_txid().to_string();
        tx.input[0].previous_output.vout = 1;
        let error = repo
            .record_btc_attempt(&scope, &tx, record(&tx))
            .unwrap_err();
        assert!(error.contains(&first_id));
        assert_eq!(repo.reserved_btc_inputs(scope.network).unwrap().len(), 1);
        repo.acknowledge_btc_attempt(&scope, &first_id).unwrap();
        repo.record_btc_attempt(&scope, &tx, record(&tx)).unwrap();
    }

    #[test]
    fn insert_and_list_round_trip() {
        let repo = TxRepository::new(make_pool());
        let chain_data = serde_json::to_value(BtcChainData {
            vsize: Some(220),
            rbf: true,
            parent_tx_id: None,
            change_value_sat: Some(1_000),
        })
        .unwrap();
        repo.insert(NewTx {
            wallet_name: "alice".into(),
            chain: BlockChain::Bitcoin,
            account_index: 0,
            tx_hash: "deadbeef".into(),
            direction: TxDirection::Outgoing,
            status: TxStatus::Pending,
            from_address: None,
            to_address: Some("bc1qabc".into()),
            amount: 10_000,
            fee: Some(250),
            block_height: None,
            chain_data,
            created_at: 1,
        })
        .unwrap();

        let rows = repo
            .list(&TxQuery {
                wallet_name: "alice".into(),
                chain: BlockChain::Bitcoin,
                account_index: 0,
                limit: None,
            })
            .unwrap();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.tx_hash, "deadbeef");
        assert_eq!(row.amount, "10000");
        assert_eq!(row.created_at, "1970-01-01T00:00:01+00:00");
        let parsed: BtcChainData = serde_json::from_str(&row.chain_data).unwrap();
        assert_eq!(parsed.vsize, Some(220));
    }

    #[test]
    fn unique_violation_on_duplicate_hash() {
        let repo = TxRepository::new(make_pool());
        let base = NewTx {
            wallet_name: "alice".into(),
            chain: BlockChain::Bitcoin,
            account_index: 0,
            tx_hash: "0xabc".into(),
            direction: TxDirection::Outgoing,
            status: TxStatus::Pending,
            from_address: Some("0xfrom".into()),
            to_address: Some("0xto".into()),
            amount: 1,
            fee: None,
            block_height: None,
            chain_data: serde_json::Value::Object(Default::default()),
            created_at: 1,
        };
        repo.insert(base.clone()).unwrap();
        let err = repo.insert(base).unwrap_err();
        assert!(err.contains("already recorded"));
    }

    #[test]
    fn update_status_marks_confirmed() {
        let chain = BlockChain::Ethereum;
        let account_index = 0;
        let repo = TxRepository::new(make_pool());
        repo.insert(NewTx {
            wallet_name: "alice".into(),
            chain,
            account_index,
            tx_hash: "0xdef".into(),
            direction: TxDirection::Outgoing,
            status: TxStatus::Pending,
            from_address: None,
            to_address: None,
            amount: 0,
            fee: None,
            block_height: None,
            chain_data: serde_json::Value::Object(Default::default()),
            created_at: 1,
        })
        .unwrap();
        let updated = repo
            .update_status(
                "alice",
                BlockChain::Ethereum,
                "0xdef",
                TxStatus::Confirmed,
                Some(123),
                Some(200),
            )
            .unwrap();
        assert_eq!(updated, 1);
        let rows = repo
            .list(&TxQuery {
                wallet_name: "alice".into(),
                chain,
                account_index,
                limit: None,
            })
            .unwrap();
        assert_eq!(rows[0].status, TxStatus::Confirmed);
        assert_eq!(rows[0].block_height, Some("123".into()));
        assert_eq!(
            rows[0].confirmed_at,
            Some("1970-01-01T00:03:20+00:00".into())
        );
    }
}
