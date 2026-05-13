import { makeAutoObservable, runInAction } from 'mobx'
import { commands, type UIConfig } from '../bindings'
import { unwrap_result } from '../lib/handle_err'
import { Unlock } from './unlock'
import { Wallet } from './wallet'

class RootStore {
  ui_config?: UIConfig
  mnemonic_wordlist: string[] = []

  readonly unlock = new Unlock()
  readonly wallet = new Wallet()

  constructor() {
    makeAutoObservable(this)
  }

  bootstrap() {
    this.request_mnemonic_list()
  }

  on_unlock() {
    this.request_config()
    this.request_prices()
  }

  async request_config() {
    const res = await commands.getConfig().then(unwrap_result)
    runInAction(() => {
      this.ui_config = res
    })
  }

  async request_prices() {
    const res = await commands.priceFeed().then(unwrap_result)
    runInAction(() => {
      this.wallet.btc.usd_price = res.btc_usd
      this.wallet.eth.usd_price = res.eth_usd
    })
  }

  async request_mnemonic_list() {
    const res = await commands.mnemonicWordlist().then(unwrap_result)
    runInAction(() => {
      this.mnemonic_wordlist = res
    })
  }
}

export const root_store = new RootStore()
