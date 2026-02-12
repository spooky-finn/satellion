import ContentCopyIcon from '@mui/icons-material/ContentCopy'
import { IconButton, type TypographyProps } from '@mui/joy'
import React from 'react'
import { P, Row } from '../shortcuts'

type Props = {
	val?: string
	n?: number
} & TypographyProps

export const CompactSrt = ({ val: address, n = 6, ...props }: Props) => {
	const [copid, setCopied] = React.useState(false)

	if (!address) return null
	if (address.length <= n * 2) {
		return <span>{address}</span>
	}

	const start = address.slice(0, n)
	const end = address.slice(-n)

	return (
		<Row alignItems={'center'}>
			<P {...props} whiteSpace={'nowrap'} fontFamily={'monospace'}>
				{start}
				<span aria-hidden="true">…</span>
				{end}
			</P>
			<IconButton
				size="sm"
				sx={{ p: 0.2, minWidth: '20px', minHeight: '20px' }}
				onClick={() => {
					navigator.clipboard.writeText(address)
					setCopied(true)
					setTimeout(() => setCopied(false), 2000)
				}}
				variant="plain"
				color={copid ? 'primary' : 'neutral'}
			>
				<ContentCopyIcon fontSize={'xs'} />
			</IconButton>
		</Row>
	)
}
