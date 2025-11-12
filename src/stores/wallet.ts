import { makeAutoObservable } from 'mobx'
import { commands, UnlockMsg } from '../bindings'
import { notifier } from '../components/notifier'
import { EthereumWallet } from '../routes/ethereum/wallet.store'
import { BitcoinWallet } from './bitcoin'

export class Wallet {
  readonly eth = new EthereumWallet()
  readonly btc = new BitcoinWallet()

  id: number | null = null
  constructor() {
    makeAutoObservable(this)
  }

  initialized: boolean = false

  init(walletId: number, unlockmsg: UnlockMsg) {
    this.id = walletId
    this.eth.address = unlockmsg.ethereum.address
    this.btc.address = unlockmsg.bitcoin.address
    this.initialized = true

    this.eth.getChainInfo()
    this.eth.getBalance(walletId)
  }

  async forget(id: number) {
    const r = await commands.forgetWallet(id)
    if (r.status === 'error') {
      notifier.err(r.error)
      return
    }
    this.initialized = false
  }

  async reset() {
    this.id = null
    this.initialized = false
  }
}
