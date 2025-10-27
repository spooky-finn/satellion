import { Button, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { route } from '../routes'
import { P, Row } from '../shortcuts'
import CreateWallet from './create_wallet'
import { PassphraseInput } from './mnemonic/create_passphrase'
import { root_store } from './store/root'

const UnlockWallet = () => {
  const { unlock } = root_store

  const navigate = useNavigate()

  useEffect(() => {
    unlock.loadAvailableWallets().then(wallets => {
      if (wallets.length === 0) {
        navigate(route.create_wallet)
      }
    })
  }, [])

  return (
    <Stack spacing={2}>
      <P level="h2">Unlock Wallet</P>
      <Stack spacing={1} width={'fit-content'}>
        {unlock.availableWallets.map(key => (
          <Button
            key={key.id}
            onClick={() => unlock.setUnlockWallet(key)}
            variant={unlock.walletToUnlock?.id === key.id ? 'soft' : 'plain'}
          >
            {key.name}
          </Button>
        ))}
      </Stack>
      {unlock.walletToUnlock && (
        <Row>
          <PassphraseInput
            placeholder={`Passphrase`}
            value={unlock.unlockPassphrase}
            onChange={e => unlock.setUnlockPassphrase(e.target.value)}
          />
          <Button
            disabled={unlock.walletToUnlock === null}
            onClick={() => {
              unlock.unlockWalletAction(root_store.wallet).then(() => {
                navigate(route.home)
              })
            }}
          >
            Unlock
          </Button>
        </Row>
      )}
      <CreateWallet />
    </Stack>
  )
}

export default observer(UnlockWallet)
