import { makeAutoObservable, runInAction } from 'mobx'
import { commands } from '../bindings'
import { unwrap_result } from '../lib/handle_err'
import { BiometricVM } from './biometric'
import { SettingsVM } from './settings'
import { Unlock } from './unlock'
import { Wallet } from './wallet'

class RootStore {
  mnemonic_wordlist: string[] = []

  readonly biometric = new BiometricVM()
  readonly unlock = new Unlock()
  readonly wallet = new Wallet()
  readonly settings = new SettingsVM(this.wallet, this.biometric)

  constructor() {
    makeAutoObservable(this)
  }

  bootstrap() {
    this.request_mnemonic_list()
  }

  on_unlock() {
    this.request_prices()
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
