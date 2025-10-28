import { makeAutoObservable } from 'mobx'
import { Unlock } from './unlock'
import { Wallet } from './wallet'

class RootStore {
  readonly unlock = new Unlock()
  readonly wallet = new Wallet()

  constructor() {
    makeAutoObservable(this)
  }
}

export const root_store = new RootStore()
