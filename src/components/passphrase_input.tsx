import { Input, type InputProps } from '@mui/joy'

export const PassphraseInput = (props: InputProps) => (
  <Input
    {...props}
    type="password"
    autoComplete="off"
    sx={{
      width: '200px',
      ...props.sx
    }}
  />
)
