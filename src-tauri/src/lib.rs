pub mod chain;
pub mod chain_trait;
pub mod codegen;
pub mod commands;
pub mod config;
pub mod db;
pub mod encryptor;
pub mod event_emitter;
pub mod mnemonic;
pub mod persistence;
pub mod repository;
pub mod schema;
pub mod session;
pub mod system;
pub mod utils;
pub mod wallet;
pub mod wallet_keeper;

// Allow `crate::btc` and `crate::eth` shorthand paths used across the codebase
use chain::{btc, eth};
