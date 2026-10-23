import { MnemonicStore } from '../mnemonic_store.svelte'

/** Coordinates creation steps; the route clears recovery secrets on unmount. */
export class CreateFlow {
  mode = $state<'choose' | 'generate' | 'verify' | 'passphrase' | 'import'>(
    'choose',
  )
  readonly mnemonic = new MnemonicStore()
  busy = $state(false)
  error = $state('')
  wordCount = $state(12)
  words = $state<string[]>(Array(12).fill(''))
  capsLock = $state(false)
  keyboardLang = $state('')
  selectedWord = $state(0)
  copied = $state(false)

  constructor(
    private readonly wordlist: () => string[],
    private readonly onSaved: () => void,
  ) {}

  get importComplete() {
    return (
      this.words.length === this.wordCount &&
      this.words.every(word => !!word && this.wordlist().includes(word))
    )
  }

  async generate() {
    this.busy = true
    this.error = ''
    this.copied = false

    try {
      await this.mnemonic.generate()

      if (this.mnemonic.mnemonic.length) this.mode = 'generate'
    } catch (error) {
      this.error = String(error)
    } finally {
      this.busy = false
    }
  }

  paste(event: ClipboardEvent) {
    const words =
      event.clipboardData?.getData('text').trim().toLowerCase().split(/\s+/) ??
      []

    if ([12, 24].includes(words.length)) {
      event.preventDefault()
      this.wordCount = words.length
      this.words = words
    }
  }

  async copyMnemonic() {
    try {
      await navigator.clipboard.writeText(this.mnemonic.mnemonic.join(' '))
      this.copied = true
    } catch (error) {
      this.error = String(error)
    }
  }

  async save() {
    if (this.busy) return

    this.busy = true
    this.error = ''

    try {
      if (this.mode === 'import')
        this.mnemonic.set_mnemonic(
          this.words.map(word => word.trim().toLowerCase()),
        )

      await this.mnemonic.createWallet(
        this.mode === 'import' ? 'Import' : 'Generation',
      )
      this.onSaved()
    } catch (error) {
      this.error = String(error)
    } finally {
      this.busy = false
    }
  }

  /** Drop recovery words and passphrases when leaving the creation screen. */
  clearSecrets() {
    this.mnemonic.mnemonic = []
    this.mnemonic.verification_words = {}
    this.mnemonic.passphrase_store.passphrase = ''
    this.mnemonic.passphrase_store.repeat_passphrase = ''
    this.words = []
  }
}
