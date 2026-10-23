import { commands } from '../bindings'
import { handle_err } from '../lib/handle_err'
import { unwrap_result } from '../lib/handle_err'
import { BiometricVM } from './biometric.svelte'
import { SettingsVM } from './settings.svelte'
import { Unlock } from './unlock.svelte'
import { Wallet } from './wallet.svelte'

/** Shared desktop state and startup requests for the current app session. */
class RootStore {
  mnemonic_wordlist = $state.raw<string[]>([])
  readonly biometric = new BiometricVM()
  readonly unlock = new Unlock()
  readonly wallet = new Wallet()
  readonly settings = new SettingsVM(this.wallet, this.biometric)

  bootstrap() {
    return Promise.all([
      this.request_mnemonic_list(),
      this.settings.request_config(),
    ])
  }

  on_unlock() {
    this.request_prices().catch(handle_err)
  }

  async request_prices() {
    const res = await commands.priceFeed().then(unwrap_result)

    this.wallet.btc.usd_price = res.btc_usd
    this.wallet.eth.usd_price = res.eth_usd
  }

  async request_mnemonic_list() {
    const res = await commands.mnemonicWordlist().then(unwrap_result)

    this.mnemonic_wordlist = res
  }
}

export const root_store = new RootStore()
