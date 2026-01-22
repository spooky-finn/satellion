import { Button, Stack } from '@mui/joy'
import { makeAutoObservable } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import { Navbar } from '../components/navbar'
import { P, Row } from '../shortcuts'
import { GenerateMnemonicFlow } from './wallet/gen/flow'
import { ImportMnemonic } from './wallet/import'

type Flow = 'import' | 'gen'

class State {
	constructor() {
		makeAutoObservable(this)
	}

	flow?: Flow
	set_flow(f: Flow) {
		this.flow = f
	}
}

export const CreateWallet = observer(() => {
	const [state] = useState(() => new State())
	switch (state.flow) {
		case 'gen':
			return <GenerateMnemonicFlow />
		case 'import':
			return <ImportMnemonic />
		default:
			return <SelectFlow state={state} />
	}
})

const SelectFlow = observer(({ state }: { state: State }) => (
	<Stack gap={2} alignItems={'center'}>
		<Navbar hideLedgers />
		<P level="h2">Add wallet</P>
		<Row sx={{ width: 'min-content' }}>
			<Button
				variant="soft"
				color="neutral"
				onClick={() => state.set_flow('import')}
			>
				Import
			</Button>
			<Button
				variant="soft"
				color="neutral"
				onClick={() => state.set_flow('gen')}
			>
				Generate
			</Button>
		</Row>
	</Stack>
))
