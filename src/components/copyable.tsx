import type { TypographyProps } from '@mui/joy'
import React from 'react'
import { notifier } from '../lib/notifier'
import { P } from '../shortcuts'

type Props = {
  val?: string
  n?: number
  copy?: boolean
} & TypographyProps

export const Copyable = ({
  val: address,
  n = 6,
  copy,
  onClick,
  sx,
  ...props
}: Props) => {
  const [copied, setCopied] = React.useState(false)
  const copiedTimer = React.useRef<ReturnType<typeof setTimeout> | null>(null)

  React.useEffect(
    () => () => {
      if (copiedTimer.current) clearTimeout(copiedTimer.current)
    },
    [],
  )

  if (!address) return null

  const handleClick: NonNullable<TypographyProps['onClick']> = e => {
    onClick?.(e)
    if (!copy || e.defaultPrevented) return

    e.stopPropagation()
    e.preventDefault()
    void navigator.clipboard.writeText(address).then(() => {
      setCopied(true)
      notifier.ok('Copied to clipboard')
      if (copiedTimer.current) clearTimeout(copiedTimer.current)
      copiedTimer.current = setTimeout(() => setCopied(false), 1500)
    })
  }

  const start = address.slice(0, n)
  const end = address.slice(-n)
  const value =
    address.length <= n * 2 ? (
      address
    ) : (
      <>
        {start}
        <span aria-hidden="true">…</span>
        {end}
      </>
    )

  const copySx = copy
    ? {
        bgcolor: copied ? 'success.softBg' : 'transparent',
        borderRadius: 'sm',
        cursor: 'copy',
        px: 0.5,
        py: 0.25,
        transition: 'background-color 180ms ease',
      }
    : undefined

  const mergedSx = [
    ...(copySx ? [copySx] : []),
    ...(Array.isArray(sx) ? sx : sx ? [sx] : []),
  ]

  return (
    <P
      {...props}
      whiteSpace={'nowrap'}
      fontFamily={'monospace'}
      onClick={handleClick}
      sx={mergedSx}
    >
      {value}
    </P>
  )
}
