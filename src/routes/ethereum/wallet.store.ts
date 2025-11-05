import { makeAutoObservable } from 'mobx'
import { Balance, ChainInfo, commands } from '../../bindings'
import { notifier } from '../../components/notifier'

export class EthereumWallet {
  constructor() {
    makeAutoObservable(this)
  }

  address!: string
  chainInfo!: ChainInfo

  balance!: Balance | null
  setBalance(b: Balance | null) {
    this.balance = b
  }

  setChainInfo(c: ChainInfo) {
    this.chainInfo = c
  }

  async getChainInfo() {
    const r = await commands.ethChainInfo()
    if (r.status === 'error') {
      notifier.err(r.error)
      return
    }
    this.setChainInfo(r.data)
  }

  async getBalance() {
    this.setBalance(null)
    const address = this.address
    const r = await commands.ethGetBalance(address)
    if (r.status === 'error') {
      notifier.err(r.error)
      return
    }
    this.setBalance(r.data)
  }
}
