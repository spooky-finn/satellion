import { Button, Container, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { MnemonicWordInput } from '../../../components/mnemonic_word_input'
import { Navbar } from '../../../components/navbar'
import { NavigateUnlock, P } from '../../../shortcuts'
import { store } from '../mnemonic_store'
import type { FlowState } from './flow_state'

export const VerifyMnemonic = observer(({ flow }: { flow: FlowState }) => (
	<Stack gap={1} alignItems={'center'}>
		<Navbar hideLedgers />
		<Container maxWidth="sm">
			<Stack gap={1} alignItems={'center'}>
				<P level="h2">Enter your mnemonic words</P>
				{store.verification_indices.map(index => (
					<MnemonicWordInput
						id={index}
						key={index}
						value={store.verification_words[index]}
						onChange={e => store.set_verification_words(index, e.target.value)}
						visible
					/>
				))}

				<Stack gap={1} alignItems={'center'}>
					<Button
						variant="soft"
						color="primary"
						sx={{ width: 'min-content' }}
						onClick={() => {
							const status = store.verify()
							if (status) {
								flow.set_stage('set_passphrase')
							}
						}}
					>
						Verify
					</Button>
					<NavigateUnlock />
				</Stack>

				{store.verification_successfull ===
				null ? null : store.verification_successfull ? (
					<P>Verification successful</P>
				) : (
					<P color="danger">Verification failed</P>
				)}
			</Stack>
		</Container>
	</Stack>
))
