/**
 * TypeScript bindings for Tauri commands
 * Generated from Rust types in src-tauri/src/commands.rs
 */

export interface SyncStatus {
  height: number
  sync_completed: boolean
}

export interface AvailableWallet {
  id: number
  name: string
}

export interface UnlockMsg {
  walletId: number
  ethereum: {
    address: string
  }
}

export interface EthereumChainInfo {
  block_number: number
  block_hash: string
  base_fee_per_gas: number | null
}
