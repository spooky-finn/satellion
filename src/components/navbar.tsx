import LockIcon from '@mui/icons-material/Lock'
import { Button } from '@mui/joy'
import { Link, useNavigate } from 'react-router'
import { route } from '../routes'
import { Row } from '../shortcuts'
import { AppMenu } from './menu'
import { ThemeSwitcher } from './theme_switcher'

export const Navbar = ({ hideLedgers }: { hideLedgers?: boolean }) => {
  const navigate = useNavigate()
  return (
    <Row justifyContent={'space-between'} width={'100%'}>
      {hideLedgers !== true && (
        <Row gap={1}>
          <BlockchainLink
            to={route.bitcoin}
            src={new URL('/bitcoin.webp', import.meta.url).toString()}
            alt="Bitcoin"
            label="Bitcoin"
          />
          <BlockchainLink
            to={route.ethereum}
            src={new URL('/ethereum.webp', import.meta.url).toString()}
            alt="Ethereum"
            label="Ethereum"
          />
        </Row>
      )}
      <Row ml="auto" gap={0}>
        <ThemeSwitcher />
        <AppMenu />
        <Button
          variant="plain"
          color="neutral"
          onClick={() => navigate(route.unlock_wallet)}
        >
          <LockIcon />
        </Button>
      </Row>
    </Row>
  )
}

const BlockchainLink = (props: {
  to: string
  src: string
  alt: string
  label: string
}) => (
  <Link to={props.to}>
    <Button
      size="sm"
      color="neutral"
      variant="soft"
      startDecorator={
        <img src={props.src} alt={props.alt} width={'auto'} height={22} />
      }
    >
      {props.label}
    </Button>
  </Link>
)
