import {
	Button,
	Divider,
	Input,
	Modal,
	ModalClose,
	ModalDialog,
} from '@mui/joy'
import { makeAutoObservable } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import { commands } from '../../bindings'
import { CompactSrt } from '../../components/compact_str'
import { NumberInput } from '../../components/number_input'
import { notifier } from '../../lib/notifier'
import { P, Row } from '../../shortcuts'

class DeriveChild {
	constructor() {
		makeAutoObservable(this)
	}
	isOpen = false
	setIsOpen(o: boolean) {
		this.isOpen = o
	}
	label?: string
	setLabel(l: string) {
		this.label = l
	}
	index?: number
	setIndex(i?: number) {
		this.index = i
	}
	address?: string
	setAddress(a: string) {
		this.address = a
	}

	async getAvaiableIndex() {
		const res = await commands.btcUnoccupiedDeriviationIndex()
		if (res.status === 'error') {
			notifier.err(res.error)
			throw Error(res.error)
		}
		this.index = res.data
	}

	async derive() {
		if (!this.label) throw Error('label is not set')
		if (!this.index) throw Error('index is not set')
		const res = await commands.btcDeriveAddress(this.label, this.index)
		if (res.status === 'error') {
			notifier.err(res.error)
			throw Error(res.error)
		}
		this.setAddress(res.data)
	}
}

export const DeriveChildAddress = observer(() => {
	const [state] = useState(() => new DeriveChild())
	return (
		<Row alignItems={'center'}>
			<Button
				size="sm"
				variant="soft"
				sx={{ width: 'fit-content' }}
				onClick={() => {
					state.setIsOpen(true)
					state.getAvaiableIndex()
				}}
			>
				Derive child
			</Button>
			<Modal open={state.isOpen} onClose={() => state.setIsOpen(false)}>
				<ModalDialog sx={{ pr: 6 }}>
					<ModalClose />
					<P level="h3">Derive child address</P>
					<Row alignItems={'center'}>
						<P>Index</P>
						<NumberInput
							size="sm"
							sx={{ maxWidth: 70 }}
							value={state.index}
							onChange={v => state.setIndex(v)}
						/>
					</Row>
					<Input
						sx={{ width: '200px' }}
						size="sm"
						placeholder="label"
						value={state.label}
						onChange={e => state.setLabel(e.target.value)}
					/>
					<Button
						sx={{ width: 'fit-content' }}
						disabled={!state.label || !state.index}
						size="sm"
						onClick={() => state.derive()}
					>
						Derive
					</Button>
					<Divider />
					<CompactSrt val={state.address} />
				</ModalDialog>
			</Modal>
		</Row>
	)
})
