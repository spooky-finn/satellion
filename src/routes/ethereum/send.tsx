import { Button, Input, Option, Select, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Navbar } from '../../components/navbar'
import { P, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'

export const EthereumSend = observer(() => {
  const { wallet } = root_store
  const { send } = root_store.wallet.eth
  return (
    <Stack gap={1}>
      <Navbar />
      <P level="h3" color="primary">
        Transact on Ethereum
      </P>
      <Input
        placeholder="Recipient address"
        value={send.address}
        onChange={e => send.setAddress(e.target.value)}
        onBlur={() => send.verifyAddress()}
        error={!!send.address && !send.isAddressValid}
      />
      <TokenSelect />
      <CurrentBalance />
      <AmountInput />
      <Button
        disabled={send.disabled}
        sx={{ width: 'min-content' }}
        size="sm"
        onClick={() => send.createTrasaction(wallet.id!)}
      >
        Estimate
      </Button>
      <TransactionCost />
    </Stack>
  )
})

const TokenSelect = observer(() => {
  const { eth } = root_store.wallet
  return (
    <Select
      placeholder="Token"
      value={eth.send.selectedToken}
      onChange={(_, value) => eth.send.setSelectedToken(value)}
    >
      {eth.tokens_with_balance.map(t => (
        <Option key={t.symbol} value={t.symbol}>
          {t.symbol}
        </Option>
      ))}
    </Select>
  )
})

const CurrentBalance = observer(() => {
  const { eth } = root_store.wallet
  const { send } = eth
  const selectedToken = send.selectedToken
  if (!selectedToken) return null
  const token = eth.balance?.tokens.find(
    token => token.symbol === selectedToken
  )
  return (
    <P>
      Balance: {token?.balance} {token?.symbol}
    </P>
  )
})

const AmountInput = observer(() => {
  const { send } = root_store.wallet.eth
  return (
    <Row>
      <Input
        placeholder="Amount"
        value={send.amount ?? ''}
        type="number"
        onChange={e => {
          const value = e.target.value
          if (value === '') {
            send.setAmount(0)
          } else {
            send.setAmount(Number(value))
          }
        }}
      />
    </Row>
  )
})

const TransactionCost = observer(() => {
  const { wallet } = root_store
  const { send } = root_store.wallet.eth
  if (!send.preconfirmInfo || !wallet.eth.price) {
    return null
  }
  const usd_tx_cost = Number(send.preconfirmInfo.cost) * wallet.eth.price
  return (
    <P>
      Cost: {send.preconfirmInfo.cost} ETH ~ {usd_tx_cost.toFixed(2)} USD
    </P>
  )
})
