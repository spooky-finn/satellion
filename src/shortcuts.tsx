import {
  Button,
  ButtonProps,
  CircularProgress,
  Stack,
  StackProps,
  Typography
} from '@mui/joy'
import { useNavigate } from 'react-router'
import { route } from './routes'

export const P = Typography
export const Row = (props: StackProps) => (
  <Stack gap={1} direction={'row'} {...props} />
)
export const LinkButton = (props: ButtonProps & { to: string }) => {
  const navigate = useNavigate()
  return <Button size="sm" {...props} onClick={() => navigate(props.to)} />
}
export const Progress = () => <CircularProgress size="sm" color="neutral" />

export const NavigateUnlock = (props: ButtonProps) => {
  const navigate = useNavigate()
  return (
    <Button
      variant="soft"
      color="neutral"
      {...props}
      onClick={() => navigate(route.unlock_wallet)}
    >
      Back to Home
    </Button>
  )
}
