import { makeAutoObservable, runInAction } from 'mobx'
import type { BitcoinUnlockDto, BlockChain } from '../../bindings'
import { commands } from '../../bindings/btc'
import { AccountSelectorVM } from '../../components/account_selector'
import { notifier } from '../../lib/notifier'
import { Loader } from '../../stores/loader'
import { UtxoListVM } from './list_utxo'
import { BitcoinTransferVM } from './transfer.vm'

export class BitcoinWalletVM {
  readonly chain: BlockChain = 'Bitcoin'
  readonly loader = new Loader()
  readonly account_selector = new AccountSelectorVM(this.chain, async _ => {
    await this.load_account_info()
  })
  readonly transfer = new BitcoinTransferVM()
  readonly utxo_list = new UtxoListVM()

  constructor() {
    makeAutoObservable(this)
  }

  init(unlock: BitcoinUnlockDto) {
    this.account_selector.init(unlock.accounts, unlock.active_account.index)
    this.init_with_account_info(unlock.active_account)
    this.utxo_list.set_utxo(unlock.active_account.utxo)
  }

  init_with_account_info(info: BitcoinUnlockDto['active_account']) {
    this.address = info.address
    this.total_balance_sat = info.total_balance
  }

  address!: string
  usd_price = 0

  warning?: string
  setWarning(w?: string) {
    this.warning = w
  }

  height?: number
  setHeight(h: number) {
    this.height = h
    this.warning = undefined
  }

  total_balance_sat: string = '0'
  set_total_balance_sat(s: string) {
    this.total_balance_sat = s
  }

  async load_account_info() {
    this.loader.start()
    const res = await commands.accountInfo()
    const utxo = await this.fetch_utxo()

    this.loader.stop()
    if (res.status === 'error') {
      notifier.err(res.error)
      throw res.error
    }

    runInAction(() => {
      this.init_with_account_info(res.data)
      this.utxo_list.set_utxo(utxo)
    })
  }

  async fetch_utxo() {
    this.loader.start()
    const syncRes = await commands.syncUtxos()
    if (syncRes.status === 'error') {
      notifier.err(syncRes.error)
      throw new Error(syncRes.error)
    }
    this.loader.stop()
    return syncRes.data
  }
}
