import { invoke } from '@tauri-apps/api/core'
import { makeAutoObservable } from 'mobx'
import { notifier } from '../../components/notifier/notifier'

class PassphraseStore {
  constructor() {
    makeAutoObservable(this)
  }
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
      const msg = 'Passphrases do not match'
      notifier.err(msg)
      throw Error(msg)
    }
  }
}

class MnemonicStore {
  readonly passphraseStore = new PassphraseStore()
  constructor() {
    makeAutoObservable(this)
  }
  walletName: string = ''
  setWalletName(walletName: string) {
    this.walletName = walletName
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

  async generate() {
    const mnemonic = await invoke('generate_mnemonic')
    if (mnemonic) {
      const words = (mnemonic as string).split(' ')
      this.setMnemonic(words)
      this.setVerificationIndices(words)
    }
  }

  verify() {
    const status = this.verificationIndices.every(
      index => this.verificationWords[index] === this.mnemonic[index]
    )
    this.verificationSuccessfull = status
    return status
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

  async createWallet() {
    try {
      return await invoke<boolean>('create_wallet', {
        mnemonic: this.mnemonic.join(' '),
        passphrase: this.passphraseStore.passphrase,
        name: this.walletName
      })
    } catch (error: any) {
      notifier.err(error)
      throw error
    }
  }
}

export const store = new MnemonicStore()
