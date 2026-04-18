import { makeAutoObservable } from 'mobx'
import type { EthereumUnlockDto } from '../../../bindings'
import {
  commands,
  type NetworkStatus,
  type WalletBalance,
} from '../../../bindings/eth'
import { notifier } from '../../../lib/notifier'
import { Loader } from '../../../view_model/loader'
import { TransferVM } from './transfer.vm'

export class EthereumWalletVM {
  readonly balance = new Loader<WalletBalance>()
  readonly transfer = new TransferVM()

  constructor() {
    makeAutoObservable(this)
  }

  init(unlock: EthereumUnlockDto) {
    this.address = unlock.address
    this.getChainInfo()
    this.getBalance()
  }

  address!: string
  chainInfo!: NetworkStatus
  usd_price = 0

  setChainInfo(c: NetworkStatus) {
    this.chainInfo = c
  }

  get tokens_with_balance() {
    return this.balance?.data?.tokens.filter(t => Number(t.balance) > 0) ?? []
  }

  async getChainInfo() {
    const r = await commands.getNetworkStatus()
    if (r.status === 'error') {
      notifier.err(r.error)
      return
    }
    this.setChainInfo(r.data)
  }

  async getBalance() {
    this.balance.start()
    const r = await commands.getWalletBalance(this.address)
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
