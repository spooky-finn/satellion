import { commands } from '../bindings'
import { Loader } from './loader.svelte'
import type { Wallet } from './wallet.svelte'

/** Wallet selection and passphrase unlock state, cleared when the session locks. */
export class Unlock {
  readonly loader = new Loader()
  private revision = 0
  is_unlocked = $state<boolean>(false)

  set_isunlocked(c: boolean) {
    this.is_unlocked = c
  }

  target_wallet = $state<string | null>(null)

  set_target_wallet(w: string) {
    this.target_wallet = w
    this.passphrase = ''
  }

  passphrase = $state<string>('')

  set_passphrase(p: string) {
    this.passphrase = p
  }

  available_wallets = $state<string[]>([])

  set_available_wallets(w: string[]) {
    this.available_wallets = w
  }

  reset() {
    this.revision++
    this.loader.reset()
    this.is_unlocked = false
    this.target_wallet = null
    this.passphrase = ''
    this.available_wallets = []
  }

  async load_available_wallets() {
    const r = await commands.getWallets()

    if (r.status === 'error') {
      throw Error(r.error)
    }

    this.available_wallets = r.data

    if (r.data.length === 1) {
      this.target_wallet = r.data[0]
    }

    return r.data
  }

  async unlock_wallet(wallet_store: Wallet) {
    const target = this.target_wallet

    if (!target || this.loader.loading)
      throw new Error('Select a wallet to unlock')

    // A reset invalidates any unlock request still waiting on the backend.
    const revision = ++this.revision

    this.loader.start()

    try {
      const result = await commands.unlockWallet(target, this.passphrase)

      if (revision !== this.revision) throw new Error('Unlock cancelled')
      if (result.status === 'error') throw new Error(result.error)

      wallet_store.init(target, result.data)
      this.is_unlocked = true

      return result.data.last_used_chain
    } finally {
      if (revision === this.revision) {
        this.loader.stop()
        this.passphrase = ''
      }
    }
  }
}
