import { makeAutoObservable } from 'mobx'
import { commands } from '../bindings'
import { notifier } from '../components/notifier'

class ChildDeriver {
  constructor() {
    makeAutoObservable(this)
  }

  index?: number
  setIndex(i?: number) {
    this.index = i
    this.address = undefined
  }

  address?: string
  setAddress(a?: string) {
    this.address = a
  }

  async derive(walletId: number) {
    if (!this.index) {
      throw Error('index not specified')
    }
    const res = await commands.btcDeriveAddress(walletId, this.index)
    if (res.status != 'ok') {
      notifier.err(res.error)
      return
    }
    this.setAddress(res.data)
    return res.data
  }
}

export class BitcoinWallet {
  readonly childDeriver = new ChildDeriver()
  constructor() {
    makeAutoObservable(this)
  }

  address!: string
}
