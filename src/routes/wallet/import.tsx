import { Button, Input, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import { MnemonicInput, MnemonicInputSt } from '../../components/mnemonic_input'
import { Navbar } from '../../components/navbar'
import { PassphraseInput } from '../../components/passphrase_input'
import { route, useNavigate } from '../../routes'
import { P } from '../../shortcuts'
import { store } from './mnemonic_store'

export const ImportMnemonic = observer(() => {
  const navigate = useNavigate()
  const [passphrase, setPassphrase] = useState('')
  const [walletName, setWalletName] = useState('')
  const [st] = useState(() => new MnemonicInputSt())

  const handleImport = async () => {
    store.set_mnemonic(st.mnemonic)
    store.passphrase_store.set_passphrase(passphrase)
    store.set_wallet_name(walletName)

    await store.createWallet('Import')
    navigate(route.unlock_wallet)
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
          autoComplete="off"
          onChange={e => setWalletName(e.target.value)}
        />
        <MnemonicInput st={st} />
        <PassphraseInput
          placeholder="Passphrase"
          value={passphrase}
          onChange={e => setPassphrase(e.target.value)}
        />
        <Button
          sx={{ width: 'min-content' }}
          onClick={handleImport}
          disabled={!st.is_input_completed}
        >
          Import
        </Button>
      </Stack>
    </Stack>
  )
})
