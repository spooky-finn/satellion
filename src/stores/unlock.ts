import { makeAutoObservable } from 'mobx'
import { commands } from '../bindings'
import { notifier } from '../components/notifier'
import { Wallet } from './wallet'

export class Unlock {
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
    const walletsInfo = r.data
    this.setAvailableWallets(walletsInfo)
    if (walletsInfo.length === 1) {
      this.setUnlockWallet(walletsInfo[0])
    }
    return walletsInfo
  }

  async unlockWallet(walletStrore: Wallet) {
    if (!this.walletToUnlock) {
      throw new Error('No wallet selected to unlock')
    }
    const walletName = this.walletToUnlock
    const r = await commands.unlockWallet(walletName, this.passphrase)
    if (r.status === 'error') {
      notifier.err(r.error)
      throw Error(r.error)
    }
    walletStrore.init(walletName, r.data)
    this.setUnlocked(true)
    return r.data.last_used_chain
  }
}
