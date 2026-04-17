import ContentCopyIcon from '@mui/icons-material/ContentCopy'
import { IconButton, type TypographyProps } from '@mui/joy'
import React from 'react'
import { notifier } from '../lib/notifier'
import { P, Row } from '../shortcuts'

type Props = {
  val?: string
  n?: number
  copy?: boolean
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
    <Row alignItems={'center'} gap={0}>
      <P {...props} whiteSpace={'nowrap'} fontFamily={'monospace'}>
        {start}
        <span aria-hidden="true">…</span>
        {end}
      </P>
      {props.copy && (
        <IconButton
          size="sm"
          sx={{ p: 0.2, minWidth: '20px', minHeight: '20px' }}
          onClick={() => {
            navigator.clipboard.writeText(address)
            setCopied(true)
            notifier.ok('Copied to clipboard')
            setTimeout(() => setCopied(false), 2000)
          }}
          variant="plain"
          color={copid ? 'primary' : 'neutral'}
        >
          <ContentCopyIcon fontSize={'xs'} />
        </IconButton>
      )}
    </Row>
  )
}
