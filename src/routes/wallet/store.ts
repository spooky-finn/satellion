import { makeAutoObservable } from 'mobx'
import { commands, MIN_PASSPHRASE_LEN } from '../../bindings'
import { notifier } from '../../components/notifier/notifier'

class PassphraseStore {
  constructor() {
    makeAutoObservable(this)
  }
  passphrase: string = ''
  repeat_passphrase: string = ''
  set_passphrase(passphrase: string) {
    this.passphrase = passphrase
  }
  set_repeat_passphrase(repeatPassphrase: string) {
    this.repeat_passphrase = repeatPassphrase
  }
  get is_something_entered() {
    return this.passphrase && this.repeat_passphrase
  }
  get is_passphrase_matched() {
    return (
      this.passphrase === this.repeat_passphrase &&
      this.passphrase.length >= MIN_PASSPHRASE_LEN
    )
  }
  get is_mismatch() {
    return this.is_something_entered && !this.is_passphrase_matched
  }
}

class MnemonicStore {
  readonly passphrase_store = new PassphraseStore()
  constructor() {
    makeAutoObservable(this)
  }
  wallet_name: string = ''
  set_wallet_name(walletName: string) {
    this.wallet_name = walletName
  }
  mnemonic: string[] = []
  set_mnemonic(mnemonic: string[]) {
    this.mnemonic = mnemonic
  }
  verification_words: Record<number, string> = {}
  set_verification_words(index: number, value: string) {
    this.verification_words[index] = value.trim()
  }
  verification_indices: number[] = []
  verification_successfull: boolean | null = null

  async generate() {
    const resp = await commands.generateMnemonic()
    if (resp.status === 'error') {
      notifier.err(resp.error)
      return
    }
    const words = resp.data.split(' ')
    this.set_mnemonic(words)
    this.setVerificationIndices(words)
  }

  verify() {
    const status = this.verification_indices.every(
      index => this.verification_words[index] === this.mnemonic[index]
    )
    this.verification_successfull = status
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
    this.verification_indices = verifyRandom
  }

  async createWallet() {
    const resp = await commands.createWallet(
      this.mnemonic.join(' '),
      this.passphrase_store.passphrase,
      this.wallet_name
    )
    if (resp.status === 'error') {
      notifier.err(resp.error)
      throw Error(resp.error)
    }
    return resp.data
  }
}

export const store = new MnemonicStore()
