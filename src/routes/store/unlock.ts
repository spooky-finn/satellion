import { invoke } from '@tauri-apps/api/core'
import { makeAutoObservable } from 'mobx'
import { AvailableWallet, UnlockMsg } from '../../types'
import { notifier } from './notifier'
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
  unlockPassphrase: string = ''
  setUnlockPassphrase(p: string) {
    this.unlockPassphrase = p
  }

  availableWallets: AvailableWallet[] = []
  setAvailableWallets(w: AvailableWallet[]) {
    this.availableWallets = w
  }

  async loadAvailableWallets() {
    const walletsInfo = await invoke<AvailableWallet[]>('get_available_wallets')
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
    const result = await invoke<UnlockMsg>('unlock_wallet', {
      walletId: this.walletToUnlock.id,
      passphrase: this.unlockPassphrase
    }).catch((error: string) => {
      notifier.err(error)
      throw error
    })

    if (result) {
      walletStrore.init(result)
      this.setUnlocked(true)
    }
  }
}
