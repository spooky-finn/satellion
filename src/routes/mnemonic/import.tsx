import { Button, Input, Stack, Textarea } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import { Navbar } from '../../components/navbar'
import { notifier } from '../../components/notifier'
import { useNavigate } from '../../routes'
import { P } from '../../shortcuts'
import { PassphraseInput } from './create_passphrase'
import { store } from './store'

const ImportMnemonic = observer(() => {
  const navigate = useNavigate()
  const [mnemonicText, setMnemonicText] = useState('')
  const [passphrase, setPassphrase] = useState('')
  const [walletName, setWalletName] = useState('')

  const handleImport = async () => {
    const words = mnemonicText
      .trim()
      .split(/\s+/)
      .filter(w => w.length > 0)
    if (words.length < 12) {
      notifier.err('Mnemonic must be between 12 words')
      return
    }

    if (words.some(w => w.length < 2)) {
      notifier.err('Invalid mnemonic format')
      return
    }

    store.setMnemonic(words)
    store.passphraseStore.setPassphrase(passphrase)
    store.setWalletName(walletName)

    await store.createWallet()
  }

  const isFormValid = () => {
    const words = mnemonicText
      .trim()
      .split(/\s+/)
      .filter(w => w.length > 0)
    return words.length >= 12 && words.length <= 24 && passphrase.length > 0
  }

  return (
    <Stack gap={3}>
      <Navbar hideLedgers />
      <P level="h2">Import wallet</P>
      <P level="body-md" color="neutral">
        Enter your existing mnemonic phrase to import your wallet
      </P>

      <Stack gap={2}>
        <Textarea
          placeholder="Enter your mnemonic phrase (12-24 words)"
          value={mnemonicText}
          onChange={e => setMnemonicText(e.target.value)}
          minRows={3}
          sx={{ width: '100%', maxWidth: '500px' }}
        />
        <Input
          sx={{ width: '200px' }}
          placeholder="Wallet name"
          value={walletName}
          onChange={e => setWalletName(e.target.value)}
        />
        <PassphraseInput
          placeholder="Passphrase"
          value={passphrase}
          onChange={e => setPassphrase(e.target.value)}
        />
        <Button
          sx={{ width: 'min-content' }}
          onClick={handleImport}
          disabled={!isFormValid()}
        >
          Import
        </Button>
      </Stack>
    </Stack>
  )
})

export default ImportMnemonic
