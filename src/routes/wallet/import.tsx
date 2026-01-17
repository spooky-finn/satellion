import { Button, Input, Stack, Textarea } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import { Navbar } from '../../components/navbar'
import { notifier } from '../../components/notifier'
import { PassphraseInput } from '../../components/passphrase_input'
import { route, useNavigate } from '../../routes'
import { P } from '../../shortcuts'
import { store } from './store'

export const ImportMnemonic = observer(() => {
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

    store.set_mnemonic(words)
    store.passphrase_store.set_passphrase(passphrase)
    store.set_wallet_name(walletName)

    await store.createWallet()
    navigate(route.unlock_wallet)
  }

  const isFormValid = () => {
    const words = mnemonicText
      .trim()
      .split(/\s+/)
      .filter(w => w.length > 0)
    return words.length >= 12 && words.length <= 24
  }

  return (
    <Stack gap={1} alignItems={'center'}>
      <Navbar hideLedgers />
      <P level="h2">Import</P>
      <Stack gap={1} alignItems={'center'}>
        <Input
          sx={{ width: '200px' }}
          placeholder="Wallet name"
          value={walletName}
          onChange={e => setWalletName(e.target.value)}
        />
        <Textarea
          autoComplete="chrome-off"
          autoCorrect="off"
          placeholder="Enter your mnemonic phrase (12 words)"
          value={mnemonicText}
          onChange={e => setMnemonicText(e.target.value)}
          minRows={3}
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
