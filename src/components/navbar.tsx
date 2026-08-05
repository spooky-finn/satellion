import LockIcon from '@mui/icons-material/Lock'
import { ToggleButtonGroup, Tooltip } from '@mui/joy'
import { useLocation, useNavigate } from 'react-router'
import { type BlockChain, commands } from '../bindings'
import { notifier } from '../lib/notifier'
import { route } from '../lib/routes'
import { SettingsLink } from '../routes/settings'
import { B, Row } from '../shortcuts'
import { ThemeSwitcher } from './theme_switcher'

const blockchains: {
  chain: BlockChain
  to: string
  src: string
}[] = [
  {
    chain: 'Bitcoin',
    to: route.bitcoin,
    src: new URL('/bitcoin.webp', import.meta.url).toString(),
  },
  {
    chain: 'Ethereum',
    to: route.ethereum,
    src: new URL('/ethereum.webp', import.meta.url).toString(),
  },
]

export const Navbar = ({ hideLedgers }: { hideLedgers?: boolean }) => {
  const navigate = useNavigate()
  const { pathname } = useLocation()
  const activeBlockchain =
    blockchains.find(({ to }) => pathname.startsWith(to))?.chain ?? null

  return (
    <Row justifyContent={'space-between'} width={'100%'}>
      {hideLedgers !== true && <BlockchainToggle value={activeBlockchain} />}
      <Row ml="auto" gap={0}>
        <ThemeSwitcher />
        <SettingsLink />
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

const BlockchainToggle = ({ value }: { value: BlockChain | null }) => {
  const navigate = useNavigate()

  return (
    <ToggleButtonGroup
      value={value}
      variant="plain"
      size="sm"
      sx={{ overflow: 'hidden', borderRadius: '20px' }}
      onChange={async (_, chain: BlockChain | null) => {
        if (!chain) return

        const res = await commands.switchBlockchain(chain)
        if (res.status === 'error') {
          notifier.err(res.error)
          return
        }

        navigate(
          blockchains.find(each => each.chain === chain)?.to ?? route.bitcoin,
        )
      }}
    >
      {blockchains.map(({ chain, src }) => (
        <B
          key={chain}
          value={chain}
          color={chain === value ? 'primary' : 'neutral'}
          startDecorator={
            <img src={src} alt={chain} width={'auto'} height={22} />
          }
        >
          {chain}
        </B>
      ))}
    </ToggleButtonGroup>
  )
}
