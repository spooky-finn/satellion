import { Input, type InputProps } from '@mui/joy'
import { useState } from 'react'

interface Props {
  value?: number | null
  placeholder?: string
  width?: number
  unit?: string
  label?: string
  onChange: (v?: number) => void
}

export function NumberInput(props: Omit<InputProps, 'onChange'> & Props) {
  const [error, setError] = useState<boolean>(false)
  return (
    <Input
      sx={{ width: props.width ? `${props.width}px` : '120px' }}
      {...props}
      type="number"
      error={error}
      value={props.value == null ? '' : props.value}
      onChange={e => {
        const num = e.target.value.trim()
        if (num === '') {
          props.onChange(undefined)
          setError(true)
        } else {
          setError(false)
          props.onChange(parseFloat(num))
        }
      }}
    />
  )
}
