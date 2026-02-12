import { makeAutoObservable, runInAction } from 'mobx'
import { commands, type UIConfig } from '../bindings'
import { notifier } from '../lib/notifier'
import { Unlock } from './unlock'
import { Wallet } from './wallet'

class RootStore {
  ui_config?: UIConfig
  readonly unlock = new Unlock()
  readonly wallet = new Wallet()

  constructor() {
    makeAutoObservable(this)
  }

  async init() {
    const res = await commands.getConfig()
    if (res.status !== 'ok') {
      notifier.err(res.error)
      return
    }
    runInAction(() => {
      this.ui_config = res.data
    })
  }
}

export const root_store = new RootStore()
