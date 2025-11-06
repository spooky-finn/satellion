import { Box, Button, Divider, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { route } from '../routes'
import { P } from '../shortcuts'
import { root_store } from '../stores/root'
import { PassphraseInput } from './mnemonic/create_passphrase'

const UnlockWallet = () => {
  const { unlock, wallet } = root_store
  const navigate = useNavigate()

  function handleUnlockWallet() {
    unlock.unlockWallet(root_store.wallet).then(() => {
      navigate(route.ethereum)
    })
  }

  function handleEnterDown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleUnlockWallet()
    }
  }

  useEffect(() => {
    wallet.reset()
    unlock.reset()

    unlock.loadAvailableWallets().then(wallets => {
      if (wallets.length === 0) {
        navigate(route.create_wallet)
      }
    })

    window.addEventListener('keydown', handleEnterDown)
    return () => {
      window.removeEventListener('keydown', handleEnterDown)
    }
  }, [])

  return (
    <Stack
      spacing={2}
      alignItems={'center'}
      width={'fit-content'}
      margin={'auto'}
    >
      <P level="h2" textAlign={'center'}>
        Unlock
      </P>
      <Divider />
      <Box
        sx={{ display: 'flex', flexWrap: 'wrap', gap: 1 }}
        width={'fit-content'}
      >
        {unlock.availableWallets.map(key => (
          <Button
            key={key.id}
            color="neutral"
            onClick={() => unlock.setUnlockWallet(key)}
            variant={
              unlock.walletToUnlock?.id === key.id ? 'solid' : 'outlined'
            }
          >
            {key.name}
          </Button>
        ))}
        <Divider orientation="vertical" />
        <Button
          size="sm"
          sx={{ width: 'min-content' }}
          variant="plain"
          color="neutral"
          onClick={() => {
            navigate(route.create_wallet)
          }}
        >
          Add
        </Button>
      </Box>
      {unlock.walletToUnlock && (
        <PassphraseInput
          autoFocus
          variant="soft"
          color="primary"
          placeholder={`Passphrase`}
          value={unlock.passphrase}
          onChange={e => unlock.setPassphrase(e.target.value)}
        />
      )}
    </Stack>
  )
}

export default observer(UnlockWallet)
