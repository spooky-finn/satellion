import { makeAutoObservable } from 'mobx'

export class BitcoinWallet {
  constructor() {
    makeAutoObservable(this)
  }

  address!: string
}
