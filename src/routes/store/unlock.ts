import { invoke } from '@tauri-apps/api/core'
import { makeAutoObservable } from 'mobx'
import { AvailableWallet } from '../../types'

export class Unlock {
  constructor() {
    makeAutoObservable(this)
  }
  unlocked: boolean = false
  setUnlocked(c: boolean) {
    this.unlocked = c
  }

  unlockWallet: AvailableWallet | null = null
  setUnlockWallet(w: AvailableWallet) {
    this.unlockWallet = w
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
  }

  async unlockWalletAction() {
    const result = await invoke('unlock_wallet', {
      wallet: this.unlockWallet,
      passphrase: this.unlockPassphrase
    })
    if (result) {
      this.unlockWallet = null
      this.setUnlocked(true)
    }
  }
}
