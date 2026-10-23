import { handle_err } from '../../../lib/handle_err'
import type { BitcoinUnlock, BlockChain } from '../../../bindings'
import {
  type ChainTip,
  commands,
  type DiscoveryReportView,
} from '../../../bindings/btc'
import { AccountSelectorVM } from '../../../components/account_selector'
import { unwrap_result } from '../../../lib/handle_err'
import { notifier } from '../../../lib/notifier'
import { Resource } from '../../../lib/resource.svelte'
import { Loader } from '../../../view_model/loader.svelte'
import { ChildAddressListVM } from './child_address_list.vm.svelte'
import { FeeBumpVM } from './fee_bump.vm.svelte'
import { TransferVM } from './transfer.vm.svelte'
import { UtxoListVM } from './utxo_list.vm.svelte'

/** Bitcoin account data and the dialogs that operate on the selected account. */
export class BitcoinWalletVM {
  readonly chain: BlockChain = 'Bitcoin'
  readonly account_selector = new AccountSelectorVM(this.chain, async _ => {
    this.address = ''
    this.total_balance_sat = '0'
    this.transfer.reset()
    this.utxo_list = new UtxoListVM()
    this.child_list = new ChildAddressListVM()
    await this.load_account_info()
  })
  readonly transfer = new TransferVM()
  utxo_list = $state.raw(new UtxoListVM())
  child_list = $state.raw(new ChildAddressListVM())
  readonly fee_bump = new FeeBumpVM(() => this.load_account_info())
  readonly account_info = new Resource(() => this._fetch_account_info())
  readonly discovery_loader = new Loader<DiscoveryReportView>()

  init(unlock: BitcoinUnlock) {
    this.account_selector.init(unlock.accounts, unlock.active_account.index)
    this.init_with_account_info(unlock.active_account)
    this.load_chain_tip().catch(handle_err)
    this.account_info.refresh().catch(handle_err)
  }

  init_with_account_info(info: BitcoinUnlock['active_account']) {
    this.address = info.address
    this.account_selector.set_account_address(info.index, info.address)
    this.total_balance_sat = info.total_balance
  }

  address = $state('')
  usd_price = $state(0)
  chain_tip = $state<ChainTip | undefined>(undefined)
  total_balance_sat = $state<string>('0')

  async load_account_info() {
    await this.account_info.refresh()
  }

  async load_chain_tip() {
    this.chain_tip = await commands.chainTip().then(unwrap_result)
  }

  async discover_wallet() {
    this.discovery_loader.start()

    try {
      const report = await commands.discoverWallet().then(unwrap_result)

      this.merge_discovered_accounts(report.accounts)
      this.discovery_loader.set(report)
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

  private accountRevision = 0

  private async _fetch_account_info(): Promise<void> {
    const revision = ++this.accountRevision
    const selected = this.account_selector.active_account
    const res = await commands.accountInfo()

    if (res.status === 'error') {
      throw new Error(res.error)
    }

    const addresses = await commands.getExternalAddresess().then(unwrap_result)

    // Account switches can finish before these requests; discard stale results.
    if (
      revision !== this.accountRevision ||
      selected !== this.account_selector.active_account ||
      res.data.index !== selected
    )
      return

    this.init_with_account_info(res.data)
    this.child_list.addresses = addresses
    this.utxo_list.sync.refresh().catch(handle_err)
  }
}
