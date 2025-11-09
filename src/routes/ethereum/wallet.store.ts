import { makeAutoObservable } from 'mobx'
import { Balance, ChainInfo, commands } from '../../bindings'
import { notifier } from '../../components/notifier'
import { EthereumSendStore } from './send.store'

export class EthereumWallet {
  readonly send = new EthereumSendStore()

  constructor() {
    makeAutoObservable(this)
  }

  address!: string
  chainInfo!: ChainInfo

  balance!: Balance | null
  setBalance(b: Balance | null) {
    this.balance = b
  }
  price!: number | null
  setPrice(v: number | null) {
    this.price = v
  }

  setChainInfo(c: ChainInfo) {
    this.chainInfo = c
  }

  get eth_balance() {
    const bigintBalance = BigInt(this.balance?.wei ?? '0')
    return bigintBalance
  }

  get tokens_with_balance() {
    return this.balance?.tokens.filter(t => Number(t.balance) > 0) ?? []
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
    this.setPrice(Number(r.data.eth_price))
  }
}
