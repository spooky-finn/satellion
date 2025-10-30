import { invoke } from '@tauri-apps/api/core'
import { makeAutoObservable } from 'mobx'
import { EthereumWallet } from '../routes/ethereum/wallet.store'
import { UnlockMsg } from '../types'
import { BitcoinWallet } from './bitcoin'

export class Wallet {
  readonly eth = new EthereumWallet()
  readonly btc = new BitcoinWallet()

  id!: number
  constructor() {
    makeAutoObservable(this)
  }

  initialized: boolean = false

  init(unlockmsg: UnlockMsg) {
    this.id = unlockmsg.wallet_id
    this.eth.address = unlockmsg.ethereum.address
    this.btc.address = unlockmsg.bitcoin.address
    this.initialized = true

    this.eth.getChainInfo()
    this.eth.getBalance()
  }

  async forget() {
    if (this.id == null) {
      throw new Error('Wallet not initialized')
    }
    await invoke('forget_wallet', { walletId: this.id })
    this.initialized = false
  }
}
