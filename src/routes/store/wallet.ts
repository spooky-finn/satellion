import { invoke } from '@tauri-apps/api/core'
import { makeAutoObservable } from 'mobx'
import { EthereumChainInfo, UnlockMsg } from '../../types'

class EthereumWallet {
  constructor() {
    makeAutoObservable(this)
  }

  address!: string
  chainInfo!: EthereumChainInfo

  async getChainInfo() {
    const chainInfo = await invoke<EthereumChainInfo>('eth_chain_info')
    this.chainInfo = chainInfo
  }
}

export class Wallet {
  readonly eth = new EthereumWallet()
  constructor() {
    makeAutoObservable(this)
  }

  initialized: boolean = false

  init(unlockmsg: UnlockMsg) {
    this.eth.address = unlockmsg.ethereum.address
    this.initialized = true

    this.eth.getChainInfo()
  }
}
