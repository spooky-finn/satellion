import { Button, Container, Input, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { MIN_PASSPHRASE_LEN } from '../../../bindings'
import { notifier } from '../../../components/notifier'
import { PassphraseInput } from '../../../components/passphrase_input'
import { route, useNavigate } from '../../../routes'
import { P } from '../../../shortcuts'
import { store } from '../mnemonic_store'

export const CreatePassphrase = observer(() => {
	const navigate = useNavigate()
	const { passphrase_store } = store
	return (
		<Stack gap={1} alignItems={'center'}>
			<P level="h2">Imagine a passphrase</P>
			<Container maxWidth="sm">
				<Stack gap={1} alignItems={'center'}>
					<Input
						sx={{ width: '200px' }}
						placeholder="Wallet name"
						value={store.wallet_name}
						onChange={e => store.set_wallet_name(e.target.value)}
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
						value={passphrase_store.passphrase}
						onChange={e => passphrase_store.set_passphrase(e.target.value)}
					/>
					<KeyboardStatus />
					<PassphraseInput
						placeholder="Repeat passphrase"
						value={passphrase_store.repeat_passphrase}
						onChange={e =>
							passphrase_store.set_repeat_passphrase(e.target.value)
						}
					/>
					{passphrase_store.is_mismatch && (
						<P level="body-xs" color="danger">
							Passphrases mismatch
						</P>
					)}
					<Button
						sx={{ width: 'min-content' }}
						disabled={!passphrase_store.is_passphrase_matched}
						onClick={() => {
							if (passphrase_store.is_mismatch) {
								notifier.err('Passphrases do not match')
							}

							store.createWallet('Generation').then(() => {
								navigate(route.unlock_wallet)
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

export const KeyboardStatus = () => {
	const [capsLock, setCapsLock] = useState(false)
	const [keyboardLang, setKeyboardLang] = useState<string>('')

	useEffect(() => {
		const handleKey = (e: KeyboardEvent) => {
			setCapsLock(e.getModifierState('CapsLock'))
			// language detection: fallback to browser language
			if (e.code.startsWith('Key')) {
				setKeyboardLang(navigator.language || 'en')
			}
		}

		window.addEventListener('keydown', handleKey)
		return () => window.removeEventListener('keydown', handleKey)
	}, [])

	return (
		<Stack direction="row" gap={1}>
			<P level="body-xs" color={capsLock ? 'warning' : 'neutral'}>
				Caps Lock: {capsLock ? 'ON' : 'OFF'}
			</P>
			<P level="body-xs">Language: {keyboardLang || 'unknown'}</P>
		</Stack>
	)
}
