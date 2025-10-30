export interface SyncStatus {
  height: number
  sync_completed: boolean
}

export interface AvailableWallet {
  id: number
  name: string
}

export interface UnlockMsg {
  wallet_id: number
  ethereum: {
    address: string
  }
  bitcoin: {
    address: string
  }
}
