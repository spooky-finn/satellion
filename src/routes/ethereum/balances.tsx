import { IconButton, Stack } from '@mui/joy'
import Decimal from 'decimal.js'
import { observer } from 'mobx-react-lite'
import { RefreshIcon } from '../../components/icons/refresh.icon'
import { P, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'
import { TokenBalance } from './types'

const Token = (props: { b: TokenBalance }) => {
  let balance = props.b.balance
  const { token_symbol, decimals, ui_precision } = props.b

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
  return (
    <Row alignItems={'center'}>
      <P level="body-xs" color="neutral">
        {token_symbol}
      </P>
      <P>{balance}</P>
    </Row>
  )
}

export const Balances = observer(() => {
  return (
    <Row alignItems={'start'}>
      <Stack>
        <Token
          b={{
            token_symbol: 'ETH',
            balance: root_store.wallet.eth.balance?.wei ?? '0',
            decimals: 18,
            ui_precision: 4
          }}
        />
        {root_store.wallet.eth.balance?.tokens.map(b => (
          <Token key={b.token_symbol} b={b} />
        ))}
      </Stack>
      <IconButton
        onClick={() => root_store.wallet.eth.getBalance()}
        variant="outlined"
      >
        <RefreshIcon />
      </IconButton>
    </Row>
  )
})
