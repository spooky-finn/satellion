import { makeAutoObservable } from 'mobx'
import { commands, UnlockMsg } from '../bindings'
import { notifier } from '../components/notifier'
import { EthereumWallet } from '../routes/ethereum/wallet.store'
import { BitcoinWallet } from './bitcoin'

export class Wallet {
  readonly eth = new EthereumWallet()
  readonly btc = new BitcoinWallet()

  name?: string
  constructor() {
    makeAutoObservable(this)
  }

  init(name: string, unlock: UnlockMsg) {
    this.name = name
    this.eth.address = unlock.ethereum.address
    this.btc.address = unlock.bitcoin.address

    this.eth.getChainInfo()
    this.eth.getBalance()
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
