import { Button, Input, Option, Select, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Navbar } from '../../components/navbar'
import { P, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'
import { useState } from 'react'
import { TransferStore } from './transfer.store'

export const EthereumTransfer = observer(() => {
  const { wallet } = root_store
  const [state] = useState(() => new TransferStore())
  return (
    <Stack gap={1}>
      <Navbar />
      <P level="h3" color="primary">
        Transact on Ethereum
      </P>
      <Input
        placeholder="Recipient address"
        value={state.address}
        onChange={e => state.setAddress(e.target.value)}
        onBlur={() => state.verifyAddress()}
        error={!!state.address && !state.isAddressValid}
      />
      <TokenSelect state={state} />
      <CurrentBalance state={state}/>
      <AmountInput state={state}/>
      {!state.preconfirmInfo && (
        <Button
          loading={state.isEstimating}
          disabled={state.disabled}
          sx={{ width: 'min-content' }}
          size="sm"
          onClick={() => state.createTrasaction(wallet.id!)}
        >
          Estimate
        </Button>
      )}
      <TransactionFee state={state} />
      <SendTransaction state={state}/>
    </Stack>
  )
})

const TokenSelect = observer(({ state }: { state: TransferStore }) => (
  <Select
    placeholder="Token"
    value={state.selectedToken}
    onChange={(_, value) => state.setSelectedToken(value ?? undefined)}
  >
    {root_store.wallet.eth.tokens_with_balance.map(t => (
      <Option key={t.symbol} value={t.symbol}>
        {t.symbol}
      </Option>
    ))}
  </Select>
))

const CurrentBalance = observer(({ state }: { state: TransferStore }) => {
  const { eth } = root_store.wallet
  const selectedToken = state.selectedToken
  if (!selectedToken) return null
  const token = eth.balance?.data?.tokens.find(
    token => token.symbol === selectedToken
  )
  return (
    <P>
      Balance: {token?.balance} {token?.symbol}
    </P>
  )
})

const AmountInput = observer(({ state }: { state: TransferStore }) => (
  <Row>
    <Input
      placeholder="Amount"
      value={state.amount ?? ''}
      type="number"
      onChange={e => {
        const num = e.target.value.trim()
        if (num === '') {
          state.setAmount(undefined)
        } else {
          state.setAmount(parseFloat(num))
        }
      }}
    />
  </Row>
))

const TransactionFee = observer(({ state }: { state: TransferStore }) => {
  const { wallet } = root_store
  if (!state.preconfirmInfo || !wallet.eth.price) {
    return null
  }
  const usd_tx_cost = Number(state.preconfirmInfo.cost) * wallet.eth.price
  return (
    <P>
      Network fee: {state.preconfirmInfo.cost} ETH ~ {usd_tx_cost.toFixed(2)} USD
    </P>
  )
})

const SendTransaction = observer(({ state }: { state: TransferStore }) => {
  const { wallet } = root_store

  if (!state.preconfirmInfo || !wallet.eth.price) {
    return null
  }

  if (state.txHash) {
    return <P>Transaction sent: hash {state.txHash}</P>
  }
  return (
    <Button
      loading={state.isSending}
      onClick={() => state.signAndSend(wallet.id!)}
      sx={{ width: 'max-content' }}
      size="sm"
    >
      Sign and send
    </Button>
  )
})
