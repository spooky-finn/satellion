import { makeAutoObservable, runInAction } from 'mobx'
import type { EthereumUnlock } from '../../../bindings'
import {
  commands,
  type NetworkStatus,
  type WalletBalance,
} from '../../../bindings/eth'
import { AccountSelectorVM } from '../../../components/account_selector'
import { unwrap_result } from '../../../lib/handle_err'
import { notifier } from '../../../lib/notifier'
import { Loader } from '../../../view_model/loader'
import { TransferVM } from './transfer.vm'

export class EthereumWalletVM {
  readonly chain = 'Ethereum' as const
  readonly balance = new Loader<WalletBalance>()
  readonly transfer = new TransferVM()
  readonly account_selector = new AccountSelectorVM(this.chain, async _ => {
    await this.load_active_account()
  })

  constructor() {
    makeAutoObservable(this)
  }

  init(unlock: EthereumUnlock) {
    this.account_selector.init(unlock.accounts, unlock.active_account.index)
    this.set_active_account(unlock.active_account.address)
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
    const address = this.address
    this.balance.start()
    const r = await commands
      .getWalletBalance(address)
      .then(unwrap_result)
      .then(r => {
        this.balance.stop()
        return r
      })
    if (this.address !== address) return
    this.balance.set(r)
  }

  async load_active_account() {
    const selectedAccount = this.account_selector.active_account
    const result = await commands.ethereumAccountInfo().then(unwrap_result)
    if (
      selectedAccount !== this.account_selector.active_account ||
      result.index !== selectedAccount
    ) {
      return
    }

    runInAction(() => this.set_active_account(result.address))
    await this.getBalance()
  }

  private set_active_account(address: string) {
    this.address = address
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
