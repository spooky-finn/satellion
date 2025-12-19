import { Button, Container, Input, InputProps, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { MIN_PASSPHRASE_LEN } from '../../bindings'
import { route, useNavigate } from '../../routes'
import { P } from '../../shortcuts'
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
  const { passphraseStore } = store
  return (
    <Stack gap={1} alignItems={'center'}>
      <P level="h2">Imagine a passphrase</P>
      <Container maxWidth="sm">
        <Stack gap={1} alignItems={'center'}>
          <Input
            sx={{ width: '200px' }}
            placeholder="Wallet name"
            value={store.walletName}
            onChange={e => store.setWalletName(e.target.value)}
          />
          <P level="body-sm">
            Passphrase should contains at least {MIN_PASSPHRASE_LEN} symbols.
            Think of your wallet passphrase as an extra lock on your wallet.
            Your recovery words are the key, and the passphrase is the secret
            code that makes the key work. For best security, keep the passphrase
            stored separately from your recovery words — ideally only in your
            mind.
          </P>
          <PassphraseInput
            placeholder={`Passphrase`}
            value={passphraseStore.passphrase}
            onChange={e => passphraseStore.setPassphrase(e.target.value)}
          />
          <PassphraseInput
            placeholder="Repeat passphrase"
            value={passphraseStore.repeatPassphrase}
            onChange={e => passphraseStore.setRepeatPassphrase(e.target.value)}
          />
          <Button
            sx={{ width: 'min-content' }}
            disabled={passphraseStore.passphrase.length < MIN_PASSPHRASE_LEN}
            onClick={() => {
              passphraseStore.verifyPassphrase()
              store.createWallet().then(() => {
                navigate(route.ethereum)
              })
            }}
          >
            Save
          </Button>
        </Stack>
      </Container>
    </Stack>
  )
})

export default CreatePassphrase
