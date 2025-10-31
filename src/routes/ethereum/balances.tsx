import { Card, IconButton, Stack } from '@mui/joy'
import Decimal from 'decimal.js'
import { observer } from 'mobx-react-lite'
import { RefreshIcon } from '../../components/icons/refresh.icon'
import { P, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'
import { TokenBalance } from './types'

const prepareTokenBalance = (b: TokenBalance): string | null => {
  let { balance, token_symbol, decimals, ui_precision } = b

  if (balance === '0') {
    return null
  }

  if (token_symbol === 'ETH' || token_symbol === 'WETH') {
    const balanceDecimal = new Decimal(balance)
    const divisor = new Decimal(10).pow(decimals)
    const convertedBalance = balanceDecimal.div(divisor)
    balance = convertedBalance.toFixed(ui_precision)
  }

  balance = Number(balance).toFixed(ui_precision)
  if (Number(balance) === 0) {
    return null
  }
  return balance
}

const Token = (props: { b: TokenBalance }) => (
  <Row alignItems={'center'}>
    <P color="neutral">{props.b.token_symbol}</P>
    <P>{props.b.balance}</P>
  </Row>
)

export const Balances = observer(() => {
  const preparedBalances =
    root_store.wallet.eth.balance?.tokens
      .map(
        (b: TokenBalance): TokenBalance => ({
          ...b,
          balance: prepareTokenBalance(b) ?? '0'
        })
      )
      .filter(b => b.balance != '0') ?? []

  return (
    <Card variant="outlined" size="sm">
      <Row alignItems={'start'} justifyContent={'space-between'}>
        <Stack>
          <Token
            b={{
              token_symbol: 'ETH',
              balance: root_store.wallet.eth.balance?.wei ?? '0',
              decimals: 18,
              ui_precision: 4
            }}
          />
          {preparedBalances.length > 0 ? (
            preparedBalances.map(b => <Token key={b.token_symbol} b={b} />)
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
