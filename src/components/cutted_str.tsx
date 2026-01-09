import type { TypographyProps } from '@mui/joy'
import { P } from '../shortcuts'

type CuttedStringProps = {
  children: string
  n?: number
} & TypographyProps

export const CuttedString = ({
  children,
  n = 6,
  ...props
}: CuttedStringProps) => {
  if (children.length <= n * 2) {
    return <span>{children}</span>
  }

  const start = children.slice(0, n)
  const end = children.slice(-n)

  return (
    <P {...props}>
      {start}
      <span aria-hidden="true">…</span>
      {end}
    </P>
  )
}
