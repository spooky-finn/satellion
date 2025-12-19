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

  initialized: boolean = false

  init(name: string, unlockmsg: UnlockMsg) {
    this.name = name
    this.eth.address = unlockmsg.ethereum.address
    this.btc.address = unlockmsg.bitcoin.address
    this.initialized = true

    this.eth.getChainInfo()
    this.eth.getBalance(name)
  }

  async forget(name: string) {
    const r = await commands.forgetWallet(name)
    if (r.status === 'error') {
      notifier.err(r.error)
      return
    }
    this.initialized = false
  }

  async reset() {
    this.name = undefined
    this.initialized = false
  }
}
