import { commands, type UnlockDto } from '../bindings'
import { notifier } from '../lib/notifier'

/** Touch ID availability, wallet preferences, and automatic prompt tracking. */
export class BiometricVM {
  is_supported = $state(false)
  busy = $state(false)
  state_loaded = $state(false)
  private enabled = $state<Record<string, boolean>>({})
  private auto_attempted = $state<string[]>([])

  is_enabled_for(wallet: string): boolean {
    return !!this.enabled[wallet]
  }

  /** Prompt once per wallet until reset, allowing passphrase fallback after cancel. */
  should_auto_prompt_for(wallet: string | null, can_prompt: boolean): boolean {
    return (
      this.state_loaded &&
      !!wallet &&
      this.is_enabled_for(wallet) &&
      can_prompt &&
      !this.auto_attempted.includes(wallet)
    )
  }

  mark_auto_attempted(wallet: string) {
    this.auto_attempted.push(wallet)
  }

  reset() {
    this.is_supported = false
    this.enabled = {}
    this.busy = false
    this.state_loaded = false
    this.auto_attempted = []
  }

  async load_for(wallets: string[]) {
    const supported = await commands.isBiometricUnlockSupported()
    const supported_ok = supported.status === 'ok' && supported.data
    const next: Record<string, boolean> = {}

    if (supported_ok) {
      await Promise.all(
        wallets.map(async name => {
          const r = await commands.isBiometricUnlockEnabled(name)

          next[name] = r.status === 'ok' && r.data
        }),
      )
    }

    this.is_supported = supported_ok
    this.enabled = next
    this.state_loaded = true
  }

  async refresh(wallet: string) {
    if (!this.is_supported) return

    const r = await commands.isBiometricUnlockEnabled(wallet)

    if (r.status === 'ok') {
      this.enabled[wallet] = r.data
    }
  }

  async enable_current(wallet_name: string): Promise<boolean> {
    return this.with_busy(async () => {
      const r = await commands.enableBiometricUnlock()

      if (r.status === 'error') {
        notifier.err(r.error)

        return false
      }

      this.enabled[wallet_name] = true

      return true
    })
  }

  async disable(wallet_name: string): Promise<boolean> {
    return this.with_busy(async () => {
      const r = await commands.disableBiometricUnlock(wallet_name)

      if (r.status === 'error') {
        notifier.err(r.error)

        return false
      }

      this.enabled[wallet_name] = false

      return true
    })
  }

  async attempt_unlock(wallet_name: string): Promise<UnlockDto | null> {
    const r = await commands.unlockWalletWithBiometric(wallet_name)

    if (r.status === 'error') {
      notifier.err(r.error)

      return null
    }

    return r.data
  }

  private async with_busy<T>(fn: () => Promise<T>): Promise<T> {
    this.busy = true

    try {
      return await fn()
    } finally {
      this.busy = false
    }
  }
}
