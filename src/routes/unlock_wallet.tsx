import { Box, Button, Divider, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { PassphraseInput } from '../components/passphrase_input'
import { route } from '../routes'
import { P, Progress } from '../shortcuts'
import { root_store } from '../stores/root'

const UnlockWallet = () => {
  const { unlock, wallet } = root_store
  const navigate = useNavigate()

  function handleUnlockWallet() {
    root_store.init()
    unlock.unlock_wallet(root_store.wallet).then(lastUsedChain => {
      navigate(lastUsedChain === 'Bitcoin' ? route.bitcoin : route.ethereum)
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

    unlock.load_available_wallets().then(wallets => {
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
        {unlock.available_wallets.map(name => (
          <Button
            key={name}
            color="neutral"
            onClick={() => unlock.set_target_wallet(name)}
            variant={unlock.target_wallet === name ? 'solid' : 'outlined'}
          >
            {name}
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
      {unlock.target_wallet &&
        (unlock.loader.loading ? (
          <>
            <Progress color="primary" />
            <P level="body-sm">Inspecting blockchain ... </P>
          </>
        ) : (
          <PassphraseInput
            autoFocus
            variant="soft"
            color="primary"
            placeholder={`Passphrase`}
            value={unlock.passphrase}
            onChange={e => unlock.set_passphrase(e.target.value)}
          />
        ))}
    </Stack>
  )
}

export default observer(UnlockWallet)
