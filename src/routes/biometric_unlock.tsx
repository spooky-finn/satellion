import FingerprintIcon from '@mui/icons-material/Fingerprint'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { route } from '../lib/routes'
import { B } from '../shortcuts'
import { root_store } from '../view_model/root'

/**
 * Wires up everything the unlock screen needs for Touch ID:
 * - clears the per-session biometric state on mount,
 * - keeps enrollment state in sync with the available-wallets list,
 * - runs the OS prompt + wallet init + navigation when invoked,
 * - auto-fires the prompt as soon as the selected wallet is eligible.
 *
 * Returns a single handler so the caller can also expose a manual button
 * fallback.
 */
export function useBiometricUnlock() {
  const { unlock, wallet, biometric } = root_store
  const navigate = useNavigate()

  useEffect(() => {
    biometric.reset()
  }, [])

  useEffect(() => {
    if (unlock.available_wallets.length > 0) {
      biometric.load_for(unlock.available_wallets)
    }
  }, [unlock.available_wallets])

  async function unlock_now() {
    const target = unlock.target_wallet
    if (!target) return
    root_store.on_unlock()
    unlock.loader.start()
    try {
      const dto = await biometric.attempt_unlock(target)
      if (!dto) return
      wallet.init(target, dto)
      unlock.set_isunlocked(true)
      navigate(
        dto.last_used_chain === 'Bitcoin' ? route.bitcoin : route.ethereum,
      )
    } finally {
      unlock.loader.stop()
    }
  }

  const can_prompt = !unlock.loader.loading && !unlock.is_unlocked
  const should_auto_prompt = biometric.should_auto_prompt_for(
    unlock.target_wallet,
    can_prompt,
  )

  useEffect(() => {
    if (should_auto_prompt && unlock.target_wallet) {
      biometric.mark_auto_attempted(unlock.target_wallet)
      unlock_now()
    }
  }, [should_auto_prompt])

  return { unlock_now }
}

/**
 * Renders a "Use Touch ID" button when the currently targeted wallet has
 * biometric unlock configured, or `null` otherwise.
 */
export const BiometricUnlockButton = observer(
  ({ onClick }: { onClick: () => void }) => {
    const { unlock, biometric } = root_store
    const target = unlock.target_wallet
    if (!target || !biometric.is_enabled_for(target)) return null
    return (
      <B
        size="sm"
        variant="soft"
        color="neutral"
        startDecorator={<FingerprintIcon />}
        onClick={onClick}
      >
        Use Touch ID
      </B>
    )
  },
)
