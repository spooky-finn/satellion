import {
  Button,
  Input,
  Option,
  Select,
  Stack,
  ToggleButtonGroup
} from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import type { FeeMode } from '../../bindings'
import { Navbar } from '../../components/navbar'
import { P, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'
import { TransferStore } from './transfer.store'
import { OpenExplorerButton } from './utils/shared'

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
        sx={{ maxWidth: '500px' }}
        value={state.address}
        onChange={e => {
          state.setAddress(e.target.value)
          state.verifyAddress()
        }}
        error={!!state.address && !state.isAddressValid}
      />
      <TokenSelect state={state} />
      <CurrentBalance state={state} />
      <AmountInput state={state} />
      <FeeModeSelect state={state} />
      <Button
        loading={state.isEstimating}
        disabled={state.disabled}
        sx={{ width: 'min-content' }}
        size="sm"
        onClick={() => state.createTrasaction(wallet.name!)}
      >
        Estimate
      </Button>
      <TransactionFee state={state} />

      {!state.txHash ? (
        <SendTransaction state={state} />
      ) : (
        <TransactionDetails state={state} />
      )}
    </Stack>
  )
})

const TokenSelect = observer(({ state }: { state: TransferStore }) => (
  <Select
    placeholder="Token"
    value={state.selectedToken}
    sx={{ width: 'fit-content' }}
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
  return (
    <P>
      Network fee: {state.preconfirmInfo.fee_ceiling} gwei ~{' '}
      {state.preconfirmInfo.fee_in_usd.toFixed(2)} USD
    </P>
  )
})

const SendTransaction = observer(({ state }: { state: TransferStore }) => {
  const { wallet } = root_store
  if (!state.preconfirmInfo || !wallet.eth.price) {
    return null
  }
  return (
    <Button
      loading={state.isSending}
      onClick={() => state.signAndSend(wallet.name!)}
      sx={{ width: 'max-content' }}
      size="sm"
    >
      Send
    </Button>
  )
})

const TransactionDetails = observer(({ state }: { state: TransferStore }) => {
  if (!state.txHash) {
    return null
  }
  return (
    <Stack>
      <P>
        Transaction hash <b>{state.txHash}</b>
      </P>
      <OpenExplorerButton path={`tx/${state.txHash}`} />
    </Stack>
  )
})

const FeeModeSelect = observer(({ state }: { state: TransferStore }) => (
  <ToggleButtonGroup
    value={state.feeMode}
    onChange={(_, v) => state.setFeeMode(v)}
  >
    <Button value={'Minimal' satisfies FeeMode}>Slow</Button>
    <Button value={'Standard' satisfies FeeMode}>Standart</Button>
    <Button value={'Increased' satisfies FeeMode}>Fast</Button>
  </ToggleButtonGroup>
))
