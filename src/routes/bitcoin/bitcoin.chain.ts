import { makeAutoObservable } from 'mobx'
import type { BitcoinUnlockDto, BlockChain } from '../../bindings'
import { commands } from '../../bindings/btc'
import { AccountSelectorVM } from '../../components/account_selector'
import { notifier } from '../../lib/notifier'

export class BitcoinChain {
  readonly chain: BlockChain = 'Bitcoin'
  readonly account_selector = new AccountSelectorVM(this.chain, async _ => {
    await this.load_account_info()
  })

  constructor() {
    makeAutoObservable(this)
  }

  init(unlock: BitcoinUnlockDto) {
    this.account_selector.init(unlock.accounts, unlock.accounts[0].index)
    this.init_with_account_info(unlock.active_account)
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
    const res = await commands.btcAccountInfo()
    if (res.status == 'error') {
      notifier.err(res.error)
      throw res.error
    }

    this.init_with_account_info(res.data)
  }
}
