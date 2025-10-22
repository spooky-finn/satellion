import { Button } from '@mui/joy'
import { invoke } from '@tauri-apps/api/core'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { route } from '../routes'
import { Row } from '../shortcuts'
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
    <div>
      <h1>Unlock Wallet</h1>
      {unlock.availableWallets.map(key => (
        <Button
          key={key.id}
          onClick={() => unlock.setUnlockWallet(key)}
          variant="plain"
        >
          {key.name}
        </Button>
      ))}
      {unlock.walletToUnlock && (
        <Row>
          <PassphraseInput
            placeholder="Passphrase"
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
            Unlock {unlock.walletToUnlock.name}
          </Button>
          <Button
            color="danger"
            size="sm"
            onClick={() => {
              invoke('delete_wallets').then(() => {
                navigate(route.create_wallet)
              })
            }}
          >
            Delete Wallets
          </Button>
        </Row>
      )}
    </div>
  )
}

export default observer(UnlockWallet)
