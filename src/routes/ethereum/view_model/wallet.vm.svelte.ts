import { handle_err } from '../../../lib/handle_err'
import type { EthereumUnlock } from '../../../bindings'
import {
  commands,
  type NetworkStatus,
  type WalletBalance,
} from '../../../bindings/eth'
import { AccountSelectorVM } from '../../../components/account_selector'
import { unwrap_result } from '../../../lib/handle_err'
import { notifier } from '../../../lib/notifier'
import { Loader } from '../../../view_model/loader.svelte'
import { TransferVM } from './transfer.vm.svelte'

/** Ethereum account balances, network details, and transfer state. */
export class EthereumWalletVM {
  readonly chain = 'Ethereum' as const
  readonly balance = new Loader<WalletBalance>()
  readonly transfer = new TransferVM()
  readonly account_selector = new AccountSelectorVM(this.chain, async _ => {
    this.address = ''
    this.balance.reset()
    this.balance_error = ''
    this.transfer.reset()
    await this.load_active_account()
  })

  init(unlock: EthereumUnlock) {
    this.account_selector.init(unlock.accounts, unlock.active_account.index)
    this.set_active_account(unlock.active_account.address)
    this.getChainInfo().catch(handle_err)
    this.getBalance().catch(handle_err)
  }

  address = $state('')
  chainInfo = $state.raw<NetworkStatus | undefined>(undefined)
  usd_price = $state(0)
  balance_error = $state('')

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

  private balanceRevision = 0

  async getBalance() {
    const address = this.address
    const revision = ++this.balanceRevision

    this.balance.start()
    this.balance_error = ''

    try {
      const balance = await commands
        .getWalletBalance(address)
        .then(unwrap_result)

      // Only the latest request for the selected address may update its balance.
      if (this.address === address && revision === this.balanceRevision)
        this.balance.set(balance)
    } catch (error) {
      if (this.address === address && revision === this.balanceRevision)
        this.balance_error = String(error)

      throw error
    } finally {
      if (revision === this.balanceRevision) this.balance.stop()
    }
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

    this.set_active_account(result.address)
    this.account_selector.set_account_address(result.index, result.address)
    await this.getBalance()
  }

  private set_active_account(address: string) {
    this.address = address
  }

  async removeTokenFromBalance(address: string) {
    if (!this.balance.data) return

    this.balance.set({
      ...this.balance.data,
      tokens: this.balance.data.tokens.filter(
        token => token.address !== address,
      ),
    })
  }
}
