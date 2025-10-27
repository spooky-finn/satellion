import { invoke } from '@tauri-apps/api/core'
import { makeAutoObservable } from 'mobx'
import { EthereumChainInfo, UnlockMsg } from '../../types'
import { notifier } from './notifier'

class EthereumWallet {
  constructor() {
    makeAutoObservable(this)
  }

  address!: string
  chainInfo!: EthereumChainInfo
  setChainInfo(c: EthereumChainInfo) {
    this.chainInfo = c
  }

  async getChainInfo() {
    this.setChainInfo(
      await invoke<EthereumChainInfo>('eth_chain_info').catch((e: string) => {
        notifier.err(e)
        throw e
      })
    )
  }
}

export class Wallet {
  readonly eth = new EthereumWallet()
  id!: number
  constructor() {
    makeAutoObservable(this)
  }

  initialized: boolean = false

  init(unlockmsg: UnlockMsg) {
    this.eth.address = unlockmsg.ethereum.address
    this.id = unlockmsg.walletId
    this.initialized = true

    this.eth.getChainInfo()
  }

  async forget() {
    await invoke('forget_wallet', { walletId: this.id })
    this.initialized = false
  }
}
