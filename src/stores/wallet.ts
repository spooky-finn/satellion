import { invoke } from '@tauri-apps/api/core'
import { makeAutoObservable } from 'mobx'
import { UnlockMsg } from '../types'
import { BitcoinWallet } from './bitcoin'
import { EthereumWallet } from './ethereum'

export class Wallet {
  readonly eth = new EthereumWallet()
  readonly btc = new BitcoinWallet()

  id!: number
  constructor() {
    makeAutoObservable(this)
  }

  initialized: boolean = false

  init(unlockmsg: UnlockMsg) {
    this.eth.address = unlockmsg.ethereum.address
    this.btc.address = unlockmsg.bitcoin.address
    this.id = unlockmsg.walletId
    this.initialized = true

    this.eth.getChainInfo()
    this.eth.getBalance()
  }

  async forget() {
    await invoke('forget_wallet', { walletId: this.id })
    this.initialized = false
  }
}
