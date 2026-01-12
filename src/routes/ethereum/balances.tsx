import { Remove } from '@mui/icons-material'
import AddIcon from '@mui/icons-material/Add'
import CachedIcon from '@mui/icons-material/Cached'
import {
  Button,
  Card,
  Grid,
  IconButton,
  Input,
  Modal,
  ModalClose,
  ModalDialog,
  Stack,
  Tooltip
} from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useState, type ChangeEvent } from 'react'
import { commands, TokenBalance, type TokenType } from '../../bindings'
import { notifier } from '../../components/notifier'
import { P, Progress, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'

export const BalanceCard = observer(() => (
  <Card variant="soft" size="sm">
    <Row alignItems={'start'} justifyContent={'space-between'}>
      <Stack>
        <Balances />
      </Stack>
      <Row alignItems={'center'} justifyContent={'end'} gap={1}>
        {root_store.ui_config?.eth_anvil && <AnvilSetBalanceButton />}
        <SpecifyTokenToTrack />
        <IconButton
          onClick={() => root_store.wallet.eth.getBalance()}
          variant="plain"
        >
          <CachedIcon />
        </IconButton>
      </Row>
    </Row>
  </Card>
))

const Token = (props: { t: TokenBalance; handleUntrack: () => void }) => {
  const { symbol, balance } = props.t
  return (
    <>
      <Grid xs={1}>
        <P color="neutral">{symbol}</P>
      </Grid>
      <Grid xs={10} px={1}>
        <P>{balance}</P>
      </Grid>
      {symbol !== 'ETH' && (
        <Grid xs={1}>
          <Tooltip title="Do not track">
            <IconButton size="sm" onClick={props.handleUntrack}>
              <Remove />
            </IconButton>
          </Tooltip>
        </Grid>
      )}
    </>
  )
}

const Balances = observer(() => {
  const { eth } = root_store.wallet
  const tokens = eth.balance.data?.tokens
  if (eth.balance.loading) return <Progress />
  if (!tokens?.length) return <P color="neutral">Tokens not found</P>

  const handleTokenUntrack = async (token_address: string) => {
    await commands.ethUntrackToken(token_address)
    eth.removeTokenFromBalance(token_address)
  }

  return (
    <Grid container>
      {tokens.map(t => (
        <Token
          key={t.symbol}
          t={t}
          handleUntrack={() => handleTokenUntrack(t.address)}
        />
      ))}
    </Grid>
  )
})

const AnvilSetBalanceButton = observer(() => {
  const handleSetAnvilBalance = async () => {
    if (!root_store.wallet.eth.address) {
      notifier.err('Wallet address not available')
      return
    }
    const res = await commands.ethAnvilSetInitialBalances(
      root_store.wallet.eth.address
    )
    if (res.status === 'error') {
      notifier.err(res.error)
    } else {
      notifier.ok(res.data)
      root_store.wallet.eth.getBalance()
    }
  }
  return (
    <Tooltip title="Set initial Anvil balances (10 ETH + 9,999,999 USDT)">
      <IconButton
        onClick={handleSetAnvilBalance}
        variant="plain"
        color="primary"
      >
        <AddIcon />
      </IconButton>
    </Tooltip>
  )
})

const SpecifyTokenToTrack = observer(() => {
  const [open, setOpen] = useState(false)
  const [data, setData] = useState<TokenType | null>(null)

  const handleAddressInput = async (e: ChangeEvent<HTMLInputElement>) => {
    const address = e.target.value
    if (address.length >= 40) {
      const res = await commands.ethTrackToken(address)
      if (res.status === 'error') {
        setOpen(false)
        notifier.err(res.error)
        throw new Error(res.error)
      }
      setData(res.data)
    }
  }

  return (
    <>
      <Button
        variant="plain"
        size="sm"
        onClick={() => {
          setOpen(true)
          setData(null)
        }}
        sx={{ width: 'max-content', fontWeight: 400 }}
        color="neutral"
      >
        Track another token
      </Button>
      <Modal open={open} onClose={() => setOpen(false)}>
        <ModalDialog sx={{ pr: 6 }}>
          <ModalClose />
          {data ? (
            <P>{data.symbol} now is trackable</P>
          ) : (
            <Input
              autoFocus
              placeholder="Token Contract Address"
              onChange={handleAddressInput}
            />
          )}
        </ModalDialog>
      </Modal>
    </>
  )
})
