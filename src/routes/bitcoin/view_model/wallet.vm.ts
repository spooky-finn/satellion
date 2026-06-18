import { makeAutoObservable, runInAction } from 'mobx'
import type { BitcoinUnlock, BlockChain } from '../../../bindings'
import { commands, type DiscoveryReportView } from '../../../bindings/btc'
import { AccountSelectorVM } from '../../../components/account_selector'
import { unwrap_result } from '../../../lib/handle_err'
import { notifier } from '../../../lib/notifier'
import { Resource } from '../../../lib/resource'
import { Loader } from '../../../view_model/loader'
import { ChildAddressListVM } from './child_address_list.vm'
import { FeeBumpVM } from './fee_bump.vm'
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
  readonly fee_bump = new FeeBumpVM(() => this.load_account_info())
  readonly account_info = new Resource(() => this._fetch_account_info())
  readonly discovery_loader = new Loader<DiscoveryReportView>()

  constructor() {
    makeAutoObservable(this)
  }

  init(unlock: BitcoinUnlock) {
    this.account_selector.init(unlock.accounts, unlock.active_account.index)
    this.init_with_account_info(unlock.active_account)
    this.account_info.refresh()
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

  async discover_wallet() {
    this.discovery_loader.start()
    try {
      const report = await commands.discoverWallet().then(unwrap_result)
      runInAction(() => {
        this.merge_discovered_accounts(report.accounts)
        this.discovery_loader.set(report)
      })
      notifier.ok(
        `Discovery added ${report.paths_added} paths and ${report.utxos_added} UTXOs`,
      )
      await this.load_account_info()
    } finally {
      this.discovery_loader.stop()
    }
  }

  private merge_discovered_accounts(account_indexes: number[]) {
    const known = new Set(this.account_selector.accounts.map(a => a.index))
    const discovered = account_indexes
      .filter(index => !known.has(index))
      .map(index => ({
        index,
        name: `Account ${index}`,
      }))

    if (!discovered.length) return

    this.account_selector.accounts = [
      ...this.account_selector.accounts,
      ...discovered,
    ].toSorted((a, b) => a.index - b.index)
  }

  private async _fetch_account_info(): Promise<void> {
    const res = await commands.accountInfo()
    if (res.status === 'error') {
      notifier.err(res.error)
      throw new Error(res.error)
    }
    const addresses = await commands.getExternalAddresess().then(unwrap_result)

    runInAction(() => {
      this.init_with_account_info(res.data)
      this.child_list.addresses = addresses
    })

    this.utxo_list.sync.refresh()
  }
}
