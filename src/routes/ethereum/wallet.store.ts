import { invoke } from '@tauri-apps/api/core'
import { makeAutoObservable } from 'mobx'
import { notifier } from '../../components/notifier'
import { Balance, Chain } from './types'

export class EthereumWallet {
  constructor() {
    makeAutoObservable(this)
  }

  address!: string
  chainInfo!: Chain

  balance!: Balance | null
  setBalance(b: Balance | null) {
    this.balance = b
  }

  setChainInfo(c: Chain) {
    this.chainInfo = c
  }

  async getChainInfo() {
    this.setChainInfo(
      await invoke<Chain>('eth_chain_info').catch((e: string) => {
        notifier.err(e)
        throw e
      })
    )
  }

  async getBalance() {
    this.setBalance(null)
    const address = this.address

    const balance = await invoke<Balance>('eth_get_balance', {
      address
    }).catch((e: string) => {
      notifier.err(e)
      throw e
    })

    this.setBalance(balance)
  }
}
