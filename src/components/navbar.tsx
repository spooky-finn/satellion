import LockIcon from '@mui/icons-material/Lock'
import { Tooltip } from '@mui/joy'
import { Link, useLocation, useNavigate } from 'react-router'
import { type BlockChain, commands } from '../bindings'
import { notifier } from '../lib/notifier'
import { route } from '../lib/routes'
import { B, Row } from '../shortcuts'
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
            chain="Bitcoin"
          />
          <BlockchainLink
            to={route.ethereum}
            src={new URL('/ethereum.webp', import.meta.url).toString()}
            chain="Ethereum"
          />
        </Row>
      )}
      <Row ml="auto" gap={0}>
        <ThemeSwitcher />
        <AppMenu />
        <Tooltip title="Lock wallet" size="sm">
          <B
            variant="plain"
            color="neutral"
            onClick={() => navigate(route.unlock_wallet)}
          >
            <LockIcon />
          </B>
        </Tooltip>
      </Row>
    </Row>
  )
}

const BlockchainLink = (props: {
  to: string
  src: string
  chain: BlockChain
}) => {
  const { pathname } = useLocation()
  const active = pathname.startsWith(props.to)
  return (
    <Link to={props.to}>
      <B
        color={active ? 'primary' : 'neutral'}
        variant={active ? 'solid' : 'soft'}
        startDecorator={
          <img src={props.src} alt={props.chain} width={'auto'} height={22} />
        }
        onClick={async () => {
          const res = await commands.switchBlockchain(props.chain)
          if (res.status === 'error') {
            notifier.err(res.error)
          }
        }}
      >
        {props.chain}
      </B>
    </Link>
  )
}
