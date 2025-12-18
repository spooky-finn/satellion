import { makeAutoObservable } from 'mobx'
import { Balance, ChainInfo, commands } from '../../bindings'
import { notifier } from '../../components/notifier'
import { Loader } from '../../stores/loader'
import { TransferStore } from './transfer.store'

export class EthereumWallet {
  readonly send = new TransferStore()
  readonly balance = new Loader<Balance>()

  constructor() {
    makeAutoObservable(this)
  }

  address!: string
  chainInfo!: ChainInfo

  price!: number | null
  setPrice(v: number | null) {
    this.price = v
  }

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

  async getBalance(walletId: number) {
    this.balance.start()
    const r = await commands.ethGetBalance(this.address, walletId)
    if (r.status === 'error') {
      notifier.err(r.error)
      return
    }
    this.balance.set(r.data)
    this.setPrice(Number(r.data.eth_price))
  }

  async removeTokenFromBalance(address: string) {
    this.balance.set({
      ...this.balance.data,
      tokens:
        this.balance.data?.tokens.filter(each => each.address != address) ?? []
    } as any)
  }
}
