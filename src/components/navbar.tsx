import { Button } from '@mui/joy'
import { Link, useNavigate } from 'react-router'
import { route } from '../routes'
import { Row } from '../shortcuts'
import { ThemeSwitcher } from './theme_switcher'

export const Navbar = ({ hideLedgers }: { hideLedgers?: boolean }) => (
  <Row justifyContent={'space-between'}>
    {hideLedgers !== true && (
      <Row gap={1}>
        <LedgerButton
          to={route.bitcoin}
          src={new URL('/bitcoin.webp', import.meta.url).toString()}
          alt="Bitcoin"
          label="Bitcoin Ledger"
        />
        <LedgerButton
          to={route.ethereum}
          src={new URL('/ethereum.webp', import.meta.url).toString()}
          alt="Ethereum"
          label="Ethereum Ledger"
        />
      </Row>
    )}
    <Row ml="auto">
      <ThemeSwitcher />
      <ExitButton />
    </Row>
  </Row>
)

const ExitButton = () => {
  const navigate = useNavigate()
  return (
    <Button
      size="sm"
      color="neutral"
      variant="soft"
      onClick={() => {
        navigate(route.unlock_wallet)
      }}
    >
      Exit
    </Button>
  )
}

const LedgerButton = (props: {
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
