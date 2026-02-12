import { makeAutoObservable } from 'mobx'
import {
  type Balance,
  type ChainInfo,
  commands,
  type UnlockMsg,
} from '../../bindings'
import { notifier } from '../../lib/notifier'
import { Loader } from '../../stores/loader'
import { TransferStore } from './transfer.store'

export class EthereumWallet {
  readonly send = new TransferStore()
  readonly balance = new Loader<Balance>()

  constructor() {
    makeAutoObservable(this)
  }

  init(unlock: UnlockMsg['ethereum']) {
    this.address = unlock.address
    this.usd_price = Number(unlock.usd_price).toFixed(0)
    this.getChainInfo()
    this.getBalance()
  }

  address!: string
  chainInfo!: ChainInfo

  usd_price!: string | null

  setChainInfo(c: ChainInfo) {
    this.chainInfo = c
  }

  get tokens_with_balance() {
    return this.balance?.data?.tokens.filter(t => Number(t.balance) > 0) ?? []
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
    this.balance.start()
    const r = await commands.ethGetBalance(this.address)
    if (r.status === 'error') {
      notifier.err(r.error)
      return
    }
    this.balance.set(r.data)
  }

  async removeTokenFromBalance(address: string) {
    this.balance.set({
      ...this.balance.data,
      tokens:
        this.balance.data?.tokens.filter(each => each.address !== address) ??
        [],
    } as any)
  }
}
