import ContentCopyIcon from '@mui/icons-material/ContentCopy'
import { IconButton, type TypographyProps } from '@mui/joy'
import React from 'react'
import { notifier } from '../lib/notifier'
import { P, Row } from '../shortcuts'

export const CopyButton = ({ val }: { val: string }) => {
  const [copied, setCopied] = React.useState(false)
  return (
    <IconButton
      size="sm"
      sx={{ p: 0.2, minWidth: '24px', minHeight: '24px' }}
      onClick={() => {
        navigator.clipboard.writeText(val)
        setCopied(true)
        notifier.ok('Copied to clipboard')
        setTimeout(() => setCopied(false), 2000)
      }}
      variant="soft"
      color={copied ? 'primary' : 'neutral'}
    >
      <ContentCopyIcon fontSize={'xs'} />
    </IconButton>
  )
}

type Props = {
  val?: string
  n?: number
  copy?: boolean
} & TypographyProps

export const CompactSrt = ({ val: address, n = 6, copy, ...props }: Props) => {
  if (!address) return null
  if (address.length <= n * 2) {
    return <P>{address}</P>
  }

  const start = address.slice(0, n)
  const end = address.slice(-n)

  return (
    <Row alignItems={'center'} gap={0.5}>
      <P {...props} whiteSpace={'nowrap'} fontFamily={'monospace'}>
        {start}
        <span aria-hidden="true">…</span>
        {end}
      </P>
      {copy && <CopyButton val={address} />}
    </Row>
  )
}
