import { invoke } from '@tauri-apps/api/core'
import { makeAutoObservable } from 'mobx'

class PassphraseStore {
  constructor() {
    makeAutoObservable(this)
  }
  error: string | null = null
  passphrase: string = ''
  repeatPassphrase: string = ''
  setPassphrase(passphrase: string) {
    this.passphrase = passphrase
  }
  setRepeatPassphrase(repeatPassphrase: string) {
    this.repeatPassphrase = repeatPassphrase
  }
  verifyPassphrase() {
    if (this.passphrase !== this.repeatPassphrase) {
      this.error = 'Passphrases do not match'
      return false
    }
    this.error = null
    return true
  }
}

class MnemonicStore {
  walletCreated: boolean = false
  error: string | null = null
  readonly passphraseStore = new PassphraseStore()
  constructor() {
    makeAutoObservable(this)
  }

  mnemonic: string[] = []
  setMnemonic(mnemonic: string[]) {
    this.mnemonic = mnemonic
  }

  verificationWords: Record<number, string> = {}
  setVirificationWords(index: number, value: string) {
    this.verificationWords[index] = value.trim()
  }

  verificationIndices: number[] = []
  verificationSuccessfull: boolean | null = null

  async generateMnemonic() {
    const mnemonic = await invoke('generate_mnemonic')
    if (mnemonic) {
      const words = (mnemonic as string).split(' ')
      this.setMnemonic(words)
      this.setVerificationIndices(words)
    }
  }

  private setVerificationIndices(words: string[]) {
    // Select random indices for verification (e.g. 3 random indices)
    const mnemonicLength = words.length
    const pickCount = Math.min(3, mnemonicLength) // pick up to 3, or fewer if not enough words
    const indices = Array.from({ length: mnemonicLength }, (_, i) => i)
    // Shuffle the indices array
    for (let i = indices.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1))
      ;[indices[i], indices[j]] = [indices[j], indices[i]]
    }
    const verifyRandom: number[] = indices.slice(0, pickCount)
    verifyRandom.sort((a, b) => a - b)
    this.verificationIndices = verifyRandom
  }

  verifyMnemonic() {
    const status = this.verificationIndices.every(
      index => this.verificationWords[index] === this.mnemonic[index]
    )
    this.verificationSuccessfull = status
    return status
  }

  createWallet() {
    return invoke('create_wallet', {
      mnemonic: this.mnemonic.join(' '),
      passphrase: this.passphraseStore.passphrase,
      name: 'My Wallet'
    })
      .then(() => {
        this.walletCreated = true
      })
      .catch(error => {
        this.error = error.message
      })
  }
}

export const store = new MnemonicStore()
