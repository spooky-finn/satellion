import { Input } from '@mui/joy'
import { P, Row } from '../shortcuts'

const INPUT_WIDTH = 120

export const MnemonicWordInput = (props: {
	id: number
	value: string
	visible: boolean
	onChange: React.ChangeEventHandler<HTMLInputElement>
	onFocus?: React.FocusEventHandler<HTMLInputElement>
	onPaste?: React.ClipboardEventHandler<HTMLInputElement>
}) => {
	const { id } = props
	return (
		<Row alignItems={'center'}>
			<P level="body-xs" width={10} textAlign={'end'}>
				{id + 1}
			</P>
			<Input
				value={props.value}
				sx={{ width: INPUT_WIDTH }}
				size="sm"
				variant="outlined"
				type={props.visible ? 'text' : 'password'}
				onFocus={props.onFocus}
				onChange={props.onChange}
				slotProps={{
					input: {
						onPaste: props.onPaste,
					},
				}}
			/>
		</Row>
	)
}
