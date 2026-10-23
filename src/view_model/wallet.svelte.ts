import { commands, type UnlockDto } from '../bindings'
import { unwrap_result } from '../lib/handle_err'
import { BitcoinWalletVM } from '../routes/bitcoin/view_model/wallet.vm.svelte'
import { EthereumWalletVM } from '../routes/ethereum/view_model/wallet.vm.svelte'

/** Owns the unlocked wallet and its Bitcoin and Ethereum view models. */
export class Wallet {
  eth = $state.raw(new EthereumWalletVM())
  btc = $state.raw(new BitcoinWalletVM())
  name = $state<string | undefined>(undefined)

  init(name: string, unlock: UnlockDto) {
    this.reset()
    this.name = name
    this.eth.init(unlock.ethereum)
    this.btc.init(unlock.bitcoin)
  }

  async forget(name: string) {
    await commands.forgetWallet(name).then(unwrap_result)
  }

  /** Replace chain models so pending requests can only update discarded state. */
  reset() {
    this.name = undefined
    this.eth = new EthereumWalletVM()
    this.btc = new BitcoinWalletVM()
  }
}
