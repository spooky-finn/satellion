import {
  type CreationFlow,
  commands,
  MIN_PASSPHRASE_LEN,
} from '../../bindings/index'
import { notifier } from '../../lib/notifier/notifier.svelte'

class PassphraseStore {
  passphrase = $state<string>('')
  repeat_passphrase = $state<string>('')

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

/** Recovery phrase generation, verification, and wallet creation inputs. */
export class MnemonicStore {
  readonly passphrase_store = new PassphraseStore()
  wallet_name = $state<string>('')

  set_wallet_name(walletName: string) {
    this.wallet_name = walletName
  }

  mnemonic = $state<string[]>([])

  set_mnemonic(mnemonic: string[]) {
    this.mnemonic = mnemonic
  }

  verification_words = $state<Record<number, string>>({})

  set_verification_words(index: number, value: string) {
    this.verification_words[index] = value.trim()
  }

  verification_indices = $state<number[]>([])
  verification_successfull = $state<boolean | null>(null)

  async generate() {
    const resp = await commands.generateMnemonic()

    if (resp.status === 'error') {
      notifier.err(resp.error)

      return
    }

    const words = resp.data.split(' ')

    this.verification_words = {}
    this.verification_successfull = null
    this.set_mnemonic(words)
    this.setVerificationIndices(words)
  }

  verify() {
    const status = this.verification_indices.every(
      index => this.verification_words[index] === this.mnemonic[index],
    )

    this.verification_successfull = status

    return status
  }

  /** Pick distinct word positions, then present them in phrase order. */
  private setVerificationIndices(words: string[]) {
    const mnemonicLength = words.length
    const pickCount = Math.min(3, mnemonicLength)
    const indices = Array.from({ length: mnemonicLength }, (_, i) => i)

    for (let i = indices.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1))
      ;[indices[i], indices[j]] = [indices[j], indices[i]]
    }

    const verifyRandom: number[] = indices.slice(0, pickCount)

    verifyRandom.sort((a, b) => a - b)
    this.verification_indices = verifyRandom
  }

  async createWallet(flow: CreationFlow) {
    const resp = await commands.createWallet(
      this.mnemonic.join(' '),
      this.passphrase_store.passphrase,
      this.wallet_name,
      flow,
    )

    if (resp.status === 'error') {
      notifier.err(resp.error)
      throw Error(resp.error)
    }

    return resp.data
  }
}
