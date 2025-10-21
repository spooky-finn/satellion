import { makeAutoObservable } from 'mobx'
import { Unlock } from './unlock'

class RootStore {
  readonly unlock = new Unlock()
  constructor() {
    makeAutoObservable(this)
  }
}

export const root_store = new RootStore()
