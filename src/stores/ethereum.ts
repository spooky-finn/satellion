import { invoke } from '@tauri-apps/api/core'
import { makeAutoObservable } from 'mobx'
import { notifier } from '../components/notifier'
import { EthereumChainInfo } from '../types'

export class EthereumWallet {
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
