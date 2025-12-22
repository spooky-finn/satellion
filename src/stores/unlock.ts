import { makeAutoObservable } from 'mobx'
import { commands } from '../bindings'
import { notifier } from '../components/notifier'
import { Loader } from './loader'
import { Wallet } from './wallet'

export class Unlock {
  readonly loader = new Loader()
  constructor() {
    makeAutoObservable(this)
  }

  unlocked: boolean = false
  setUnlocked(c: boolean) {
    this.unlocked = c
  }

  walletToUnlock: string | null = null
  setUnlockWallet(w: string) {
    this.walletToUnlock = w
  }
  passphrase: string = ''
  setPassphrase(p: string) {
    this.passphrase = p
  }

  availableWallets: string[] = []
  setAvailableWallets(w: string[]) {
    this.availableWallets = w
  }

  reset() {
    this.unlocked = false
    this.walletToUnlock = null
    this.passphrase = ''
    this.availableWallets = []
  }

  async loadAvailableWallets() {
    const r = await commands.listWallets()
    if (r.status === 'error') {
      notifier.err(r.error)
      throw Error(r.error)
    }
    this.setAvailableWallets(r.data)
    if (r.data.length === 1) {
      this.setUnlockWallet(r.data[0])
    }
    return r.data
  }

  async unlockWallet(walletStrore: Wallet) {
    if (!this.walletToUnlock) {
      throw new Error('No wallet selected to unlock')
    }
    this.loader.start()
    const walletName = this.walletToUnlock
    const r = await commands
      .unlockWallet(walletName, this.passphrase)
      .finally(() => this.loader.stop())

    if (r.status === 'error') {
      notifier.err(r.error)
      this.setPassphrase('')
      throw Error(r.error)
    }
    walletStrore.init(walletName, r.data)
    this.setUnlocked(true)
    return r.data.last_used_chain
  }
}
