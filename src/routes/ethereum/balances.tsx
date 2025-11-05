import { Card, IconButton, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { TokenBalance } from '../../bindings'
import { RefreshIcon } from '../../components/icons/refresh.icon'
import { P, Row } from '../../shortcuts'
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

const Token = (props: { b: TokenBalance }) => (
  <Row alignItems={'center'}>
    <P color="neutral">{props.b.symbol}</P>
    <P>{props.b.balance}</P>
  </Row>
)

export const Balances = observer(() => {
  const tokens =
    root_store.wallet.eth.tokens_with_balance
      .map((b: TokenBalance) => ({
        ...b,
        balance: fmtBalance(b) ?? '0'
      }))
      .filter(b => b.balance != '0') ?? []

  return (
    <Card variant="outlined" size="sm">
      <Row alignItems={'start'} justifyContent={'space-between'}>
        <Stack>
          {tokens.length > 0 ? (
            tokens.map(b => <Token key={b.symbol} b={b} />)
          ) : (
            <P color="neutral">Tokens not found</P>
          )}
        </Stack>
        <IconButton
          onClick={() => root_store.wallet.eth.getBalance()}
          variant="outlined"
        >
          <RefreshIcon />
        </IconButton>
      </Row>
    </Card>
  )
})
