import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import { FlowState } from './flow_state'
import { GenerateMnemonic } from './mnemonic_generate'
import { VerifyMnemonic } from './mnemonic_verify'
import { CreatePassphrase } from './passphrase_create'

export const GenerateMnemonicFlow = observer(() => {
	const [flow] = useState(() => new FlowState())

	switch (flow.stage) {
		case 'select_mnemonic':
			return <GenerateMnemonic flow={flow} />
		case 'verify_mnemonic':
			return <VerifyMnemonic flow={flow} />
		case 'set_passphrase':
			return <CreatePassphrase />
	}
})
