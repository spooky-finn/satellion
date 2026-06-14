import { makeAutoObservable, runInAction } from 'mobx'
import { commands, type UnlockDto } from '../bindings'
import { notifier } from '../lib/notifier'

/**
 * Owns every biometric-unlock concern: platform support detection, per-wallet
 * enrollment state, the unlock/enable/disable IPC calls, and the per-session
 * auto-prompt bookkeeping consumed by the unlock screen.
 *
 * All other view models talk to biometric features through this class —
 * they never reach into `commands.*Biometric*` directly.
 */
export class BiometricVM {
  is_supported = false
  busy = false
  /** True once `load_for` has finished — auto-prompt logic waits for this. */
  state_loaded = false
  /** Per-wallet enrolled flag. Absent key = unknown / not loaded. */
  private enabled: Record<string, boolean> = {}
  /** Wallets we've already auto-prompted in this unlock session. Prevents prompt loops on cancel. */
  private auto_attempted: Set<string> = new Set()

  constructor() {
    makeAutoObservable(this)
  }

  is_enabled_for(wallet: string): boolean {
    return !!this.enabled[wallet]
  }

  /** True when the unlock screen should fire the OS prompt without user interaction. */
  should_auto_prompt_for(wallet: string | null, can_prompt: boolean): boolean {
    return (
      this.state_loaded &&
      !!wallet &&
      this.is_enabled_for(wallet) &&
      can_prompt &&
      !this.auto_attempted.has(wallet)
    )
  }

  mark_auto_attempted(wallet: string) {
    this.auto_attempted.add(wallet)
  }

  reset() {
    this.is_supported = false
    this.enabled = {}
    this.busy = false
    this.state_loaded = false
    this.auto_attempted = new Set()
  }

  /** Refresh platform support + enrolled flag for every supplied wallet. */
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
    runInAction(() => {
      this.is_supported = supported_ok
      this.enabled = next
      this.state_loaded = true
    })
  }

  /** Re-fetch the enrolled flag for a single wallet. */
  async refresh(wallet: string) {
    if (!this.is_supported) return
    const r = await commands.isBiometricUnlockEnabled(wallet)
    if (r.status === 'ok') {
      runInAction(() => {
        this.enabled[wallet] = r.data
      })
    }
  }

  /** Enroll the currently unlocked wallet. The backend reads the passphrase from the active session. */
  async enable_current(wallet_name: string): Promise<boolean> {
    return this.with_busy(async () => {
      const r = await commands.enableBiometricUnlock()
      if (r.status === 'error') {
        notifier.err(r.error)
        return false
      }
      runInAction(() => {
        this.enabled[wallet_name] = true
      })
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
      runInAction(() => {
        this.enabled[wallet_name] = false
      })
      return true
    })
  }

  /** Trigger the OS biometric prompt and, on success, return the decrypted wallet state. */
  async attempt_unlock(wallet_name: string): Promise<UnlockDto | null> {
    const r = await commands.unlockWalletWithBiometric(wallet_name)
    if (r.status === 'error') {
      notifier.err(r.error)
      return null
    }
    return r.data
  }

  private async with_busy<T>(fn: () => Promise<T>): Promise<T> {
    runInAction(() => {
      this.busy = true
    })
    try {
      return await fn()
    } finally {
      runInAction(() => {
        this.busy = false
      })
    }
  }
}
