import { Add } from '@mui/icons-material'
import KeyIcon from '@mui/icons-material/Key'
import { Box, IconButton, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { PassphraseInput } from '../components/passphrase_input'
import { ThemeSwitcher } from '../components/theme_switcher'
import { useKeyDown } from '../components/use_key_down'
import { useKeyboardRefetch } from '../components/use_keyboard_refetch'
import { route } from '../lib/routes'
import { B, P, Progress } from '../shortcuts'
import { root_store } from '../view_model/root'
import { BiometricUnlockButton, useBiometricUnlock } from './biometric_unlock'

const UnlockWallet = () => {
  const { unlock, wallet } = root_store
  const navigate = useNavigate()
  const biometric_flow = useBiometricUnlock()

  function handleUnlockWallet() {
    root_store.on_unlock()
    unlock.unlock_wallet(wallet).then(lastUsedChain => {
      navigate(lastUsedChain === 'Bitcoin' ? route.bitcoin : route.ethereum)
    })
  }

  useEffect(() => {
    wallet.reset()
    unlock.reset()

    unlock.load_available_wallets().then(wallets => {
      if (wallets.length === 0) {
        navigate(route.create_wallet)
      }
    })
  }, [])

  useKeyDown('Enter', handleUnlockWallet)

  useKeyboardRefetch(async () => {
    unlock.load_available_wallets()
  })

  return (
    <Box sx={{ position: 'relative', width: '100%', height: '100%' }}>
      <Stack
        spacing={2}
        alignItems={'center'}
        width={'fit-content'}
        margin={'auto'}
        paddingTop={8}
      >
        <Stack spacing={1} alignItems="center">
          <KeyIcon sx={{ fontSize: 64 }} />
          <P level="h2">{unlock.target_wallet}</P>
        </Stack>
        {!unlock.target_wallet && (
          <P level="body-xs" sx={{ color: 'text.tertiary' }}>
            select a wallet
          </P>
        )}
        {unlock.target_wallet &&
          (unlock.loader.loading ? (
            <>
              <Progress />
              <P level="body-sm">Inspecting blockchain ... </P>
            </>
          ) : (
            <Stack spacing={1} alignItems="center" width="100%">
              <PassphraseInput
                key={unlock.target_wallet}
                autoFocus
                variant="soft"
                color="primary"
                placeholder={`Passphrase`}
                value={unlock.passphrase}
                onChange={e => unlock.set_passphrase(e.target.value)}
              />
              <BiometricUnlockButton onClick={biometric_flow.unlock_now} />
            </Stack>
          ))}
      </Stack>
      <Box
        sx={{
          position: 'fixed',
          bottom: 16,
          display: 'flex',
          flexWrap: 'wrap',
          gap: 1,
          width: '100%',
          justifyContent: 'center',
          alignItems: 'center',
        }}
      >
        {unlock.available_wallets.map(name => (
          <B
            key={name}
            size="sm"
            color="neutral"
            onClick={() => {
              unlock.set_target_wallet(name)
            }}
            variant={unlock.target_wallet === name ? 'solid' : 'outlined'}
          >
            {name}
          </B>
        ))}
        <IconButton
          size="sm"
          variant="plain"
          color="neutral"
          onClick={() => {
            navigate(route.create_wallet)
          }}
        >
          <Add />
        </IconButton>
        <ThemeSwitcher />
      </Box>
    </Box>
  )
}

export default observer(UnlockWallet)
