import CachedIcon from '@mui/icons-material/Cached'
import {
  Button,
  Card,
  IconButton,
  Modal,
  ModalClose,
  ModalDialog,
  Stack
} from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import { TokenBalance } from '../../bindings'
import { P, Progress, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'

const fmtBalance = (b: TokenBalance): string | null => {
  if (b.balance === '0') {
    return null
  }
  const balance = Number(b.balance).toFixed(b.ui_precision)
  if (Number(balance) === 0) {
    return null
  }
  return balance
}

const Token = (props: { t: TokenBalance }) => (
  <Row alignItems={'center'}>
    <P color="neutral">{props.t.symbol}</P>
    <P>{props.t.balance}</P>
  </Row>
)

export const BalanceCard = observer(() => (
  <Card variant="outlined" size="sm">
    <Row alignItems={'start'} justifyContent={'space-between'}>
      <Stack>
        <Balances />
      </Stack>
      <Row alignItems={'center'} justifyContent={'end'}>
        <SpecifyTokenToTrack />
        <IconButton
          onClick={() =>
            root_store.wallet.eth.getBalance(root_store.wallet.id!)
          }
          variant="plain"
        >
          <CachedIcon />
        </IconButton>
      </Row>
    </Row>
  </Card>
))

const Balances = observer(() => {
  const { eth } = root_store.wallet
  const tokens =
    eth.tokens_with_balance
      .map((b: TokenBalance) => ({
        ...b,
        balance: fmtBalance(b) ?? '0'
      }))
      .filter(b => b.balance != '0') ?? []
  if (eth.balance.loading) return <Progress />
  if (!tokens.length) return <P color="neutral">Tokens not found</P>
  return tokens.map(t => <Token key={t.symbol} t={t} />)
})

const SpecifyTokenToTrack = observer(() => {
  const [open, setOpen] = useState(false)
  return (
    <>
      <Button
        variant="plain"
        size="sm"
        onClick={() => setOpen(true)}
        sx={{ width: 'max-content', fontWeight: 400 }}
        color="neutral"
      >
        Track another token
      </Button>
      <Modal open={open} onClose={() => setOpen(false)}>
        <ModalDialog>
          <ModalClose />
        </ModalDialog>
      </Modal>
    </>
  )
})
