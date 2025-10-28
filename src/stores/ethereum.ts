import { invoke } from '@tauri-apps/api/core'
import { makeAutoObservable } from 'mobx'
import { notifier } from '../components/notifier'
import { EthereumChainInfo } from '../types'
import { hexToDecimal } from '../utils/ethereum'

export class EthereumWallet {
  constructor() {
    makeAutoObservable(this)
  }

  address!: string
  chainInfo!: EthereumChainInfo

  balance!: string | null
  setBalance(b: string | null) {
    this.balance = b
  }

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

  async getBalance() {
    this.setBalance(null)
    const address = this.address

    const balance = await invoke<string>('eth_get_balance', { address })
      .catch((e: string) => {
        notifier.err(e)
        throw e
      })
      .then(b => hexToDecimal(b))

    this.setBalance(balance)
  }
}
