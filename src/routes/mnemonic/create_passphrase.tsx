import { Button, Input, InputProps, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { route, useNavigate } from '../../routes'
import { store } from './store'

export const PassphraseInput = (props: InputProps) => (
  <Input
    {...props}
    type="password"
    autoComplete="off"
    sx={{
      width: '200px',
      ...props.sx
    }}
  />
)

const CreatePassphrase = observer(() => {
  const navigate = useNavigate()
  return (
    <Stack gap={1}>
      <Input
        sx={{ width: '200px' }}
        placeholder="Wallet name"
        value={store.walletName}
        onChange={e => store.setWalletName(e.target.value)}
      />
      <PassphraseInput
        placeholder="Passphrase"
        value={store.passphraseStore.passphrase}
        onChange={e => store.passphraseStore.setPassphrase(e.target.value)}
      />
      <PassphraseInput
        placeholder="Repeat passphrase"
        value={store.passphraseStore.repeatPassphrase}
        onChange={e =>
          store.passphraseStore.setRepeatPassphrase(e.target.value)
        }
      />
      <Button
        sx={{ width: 'min-content' }}
        onClick={() => {
          store.passphraseStore.verifyPassphrase()
          store.createWallet().then(() => {
            navigate(route.home)
          })
        }}
      >
        Save
      </Button>
    </Stack>
  )
})

export default CreatePassphrase
