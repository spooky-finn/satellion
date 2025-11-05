import { makeAutoObservable } from 'mobx'
import { AvailableWallet, commands } from '../bindings'
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

  walletToUnlock: AvailableWallet | null = null
  setUnlockWallet(w: AvailableWallet) {
    this.walletToUnlock = w
  }
  passphrase: string = ''
  setPassphrase(p: string) {
    this.passphrase = p
  }

  availableWallets: AvailableWallet[] = []
  setAvailableWallets(w: AvailableWallet[]) {
    this.availableWallets = w
  }

  async loadAvailableWallets() {
    const r = await commands.getAvailableWallets()
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

  async unlockWalletAction(walletStrore: Wallet) {
    if (!this.walletToUnlock) {
      throw new Error('No wallet selected to unlock')
    }
    const walletId = this.walletToUnlock.id
    const r = await commands.unlockWallet(walletId, this.passphrase)
    if (r.status === 'error') {
      notifier.err(r.error)
      throw Error(r.error)
    }
    walletStrore.init(walletId, r.data)
    this.setUnlocked(true)
  }
}
