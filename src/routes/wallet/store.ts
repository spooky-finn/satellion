import { makeAutoObservable } from 'mobx'
import { commands, MIN_PASSPHRASE_LEN } from '../../bindings'
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
  isSomethingEntered() {
    return this.passphrase && this.repeatPassphrase
  }
  matched() {
    return (
      this.passphrase === this.repeatPassphrase &&
      this.passphrase.length >= MIN_PASSPHRASE_LEN
    )
  }
  mismatch() {
    return this.isSomethingEntered() && !this.matched()
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
    const resp = await commands.generateMnemonic()
    if (resp.status === 'error') {
      notifier.err(resp.error)
      return
    }
    const words = resp.data.split(' ')
    this.setMnemonic(words)
    this.setVerificationIndices(words)
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
    const resp = await commands.createWallet(
      this.mnemonic.join(' '),
      this.passphraseStore.passphrase,
      this.walletName
    )
    if (resp.status === 'error') {
      notifier.err(resp.error)
      throw Error(resp.error)
    }
    return resp.data
  }
}

export const store = new MnemonicStore()
