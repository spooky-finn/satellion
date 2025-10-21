import { Button } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { Row } from '../shortcuts'
import { PassphraseInput } from './mnemonic/create_passphrase'
import { root_store } from './store/root'

const UnlockWallet = () => {
  const { unlock } = root_store

  useEffect(() => {
    unlock.loadAvailableWallets()
  }, [])

  return (
    <div>
      <h1>Satellion Wallet</h1>
      {unlock.availableWallets.map(key => (
        <Button
          key={key.id}
          onClick={() => unlock.setUnlockWallet(key)}
          variant="plain"
        >
          {key.name}
        </Button>
      ))}
      {unlock.unlockWallet && (
        <Row>
          <PassphraseInput
            placeholder="Passphrase"
            value={unlock.unlockPassphrase}
            onChange={e => unlock.setUnlockPassphrase(e.target.value)}
          />
          <Button
            disabled={unlock.unlockWallet === null}
            onClick={() => {
              unlock.unlockWalletAction()
            }}
          >
            Unlock {unlock.unlockWallet.name}
          </Button>
        </Row>
      )}
    </div>
  )
}

export default observer(UnlockWallet)
