import type { Unlock } from '../../../view_model/unlock.svelte'
import type { BiometricVM } from '../../../view_model/biometric.svelte'
import type { Wallet } from '../../../view_model/wallet.svelte'
import { handle_err } from '../../../lib/handle_err'

/** Route-local login lifecycle; session state remains in the shared wallet stores. */
export class UnlockPage {
  loading = $state(true)
  error = $state('')
  private disposed = false

  constructor(
    readonly unlock: Unlock,
    readonly biometric: BiometricVM,
    readonly wallet: Wallet,
    private readonly onComplete: (chain: string) => void,
    private readonly onEmpty: () => void,
  ) {}

  start() {
    this.wallet.reset()
    this.unlock.reset()
    this.biometric.reset()
    this.load().catch(handle_err)
  }

  dispose() {
    this.disposed = true
    this.unlock.set_passphrase('')
  }

  async load() {
    this.loading = true
    this.error = ''

    try {
      const selected = this.unlock.target_wallet
      const wallets = await this.unlock.load_available_wallets()

      if (
        this.unlock.target_wallet &&
        !wallets.includes(this.unlock.target_wallet)
      )
        this.unlock.target_wallet = null
      if (this.unlock.target_wallet !== selected) this.unlock.set_passphrase('')
      if (this.disposed) return

      if (!wallets.length) {
        this.onEmpty()

        return
      }

      await this.biometric.load_for(wallets)

      if (this.disposed) return

      this.loading = false
      await this.autoPrompt()
    } catch (cause) {
      this.error = String(cause)
    } finally {
      this.loading = false
    }
  }

  private complete(chain: string) {
    if (this.disposed) return

    this.unlock.set_passphrase('')
    this.onComplete(chain)
  }

  async passwordUnlock() {
    if (
      !this.unlock.target_wallet ||
      !this.unlock.passphrase ||
      this.unlock.loader.loading
    )
      return

    try {
      this.complete(await this.unlock.unlock_wallet(this.wallet))
    } catch (cause) {
      this.error = String(cause)
    }
  }

  async biometricUnlock() {
    const target = this.unlock.target_wallet

    if (!target || this.unlock.loader.loading) return

    this.unlock.loader.start()

    try {
      const result = await this.biometric.attempt_unlock(target)

      if (result && !this.disposed) {
        this.wallet.init(target, result)
        this.unlock.set_isunlocked(true)
        this.complete(result.last_used_chain)
      }
    } catch (cause) {
      this.error = String(cause)
    } finally {
      this.unlock.loader.stop()
    }
  }

  private async autoPrompt() {
    const target = this.unlock.target_wallet

    if (
      this.biometric.should_auto_prompt_for(
        target,
        !this.unlock.loader.loading && !this.unlock.is_unlocked,
      ) &&
      target
    ) {
      this.biometric.mark_auto_attempted(target)
      await this.biometricUnlock()
    }
  }

  selectWallet(name: string) {
    this.unlock.set_target_wallet(name)
    this.error = ''
    this.autoPrompt().catch(handle_err)
  }

  keyboard(event: KeyboardEvent) {
    const refresh =
      event.key === 'F5' ||
      ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'r')

    if (!refresh) return

    event.preventDefault()

    if (!this.loading && !this.unlock.loader.loading)
      this.load().catch(handle_err)
  }
}
