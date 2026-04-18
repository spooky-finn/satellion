import { makeAutoObservable } from 'mobx'
import { commands, type UnlockDto } from '../bindings'
import { notifier } from '../lib/notifier'
import { BitcoinWalletVM } from '../routes/bitcoin/view_model/wallet.vm'
import { EthereumWalletVM } from '../routes/ethereum/view_model/wallet.vm'

export class Wallet {
  readonly eth = new EthereumWalletVM()
  readonly btc = new BitcoinWalletVM()

  name?: string
  constructor() {
    makeAutoObservable(this)
  }

  init(name: string, unlock: UnlockDto) {
    this.name = name
    this.eth.init(unlock.ethereum)
    this.btc.init(unlock.bitcoin)
  }

  async forget(name: string) {
    const r = await commands.forgetWallet(name)
    if (r.status === 'error') {
      notifier.err(r.error)
      return
    }
  }

  async reset() {
    this.name = undefined
  }
}
