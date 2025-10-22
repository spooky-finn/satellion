import { makeAutoObservable } from 'mobx'
import { UnlockMsg } from '../../types'

class EthereumWallet {
  constructor() {
    makeAutoObservable(this)
  }

  address!: string
}

export class Wallet {
  readonly eth = new EthereumWallet()
  constructor() {
    makeAutoObservable(this)
  }

  initialized: boolean = false

  init(unlockmsg: UnlockMsg) {
    this.eth.address = unlockmsg.ethereum.address
    this.initialized = true
  }
}
