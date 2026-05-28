import { makeAutoObservable, runInAction } from 'mobx'
import type { BitcoinUnlock, BlockChain } from '../../../bindings'
import { commands } from '../../../bindings/btc'
import { AccountSelectorVM } from '../../../components/account_selector'
import { unwrap_result } from '../../../lib/handle_err'
import { notifier } from '../../../lib/notifier'
import { Resource } from '../../../lib/resource'
import { ChildAddressListVM } from './child_address_list.vm'
import { TransferVM } from './transfer.vm'
import { UtxoListVM } from './utxo_list.vm'

export class BitcoinWalletVM {
  readonly chain: BlockChain = 'Bitcoin'
  readonly account_selector = new AccountSelectorVM(this.chain, async _ => {
    await this.load_account_info()
  })
  readonly transfer = new TransferVM()
  readonly utxo_list = new UtxoListVM()
  readonly child_list = new ChildAddressListVM()
  readonly account_info = new Resource(() => this._fetch_account_info())

  constructor() {
    makeAutoObservable(this)
  }

  init(unlock: BitcoinUnlock) {
    this.account_selector.init(unlock.accounts, unlock.active_account.index)
    this.init_with_account_info(unlock.active_account)
  }

  init_with_account_info(info: BitcoinUnlock['active_account']) {
    this.address = info.address
    this.total_balance_sat = info.total_balance
  }

  address!: string
  usd_price = 0

  height?: number
  total_balance_sat: string = '0'

  async load_account_info() {
    await this.account_info.refresh()
  }

  private async _fetch_account_info(): Promise<void> {
    const res = await commands.accountInfo()
    if (res.status === 'error') {
      notifier.err(res.error)
      throw new Error(res.error)
    }
    const utxo = await this.fetch_utxo()
    const addresses = await commands.getExternalAddresess().then(unwrap_result)

    runInAction(() => {
      this.init_with_account_info(res.data)
      this.utxo_list.utxo = utxo
      this.child_list.addresses = addresses
    })
  }

  async fetch_utxo() {
    const syncRes = await commands.syncUtxos()
    if (syncRes.status === 'error') {
      notifier.err(syncRes.error)
      throw new Error(syncRes.error)
    }
    return syncRes.data
  }
}
