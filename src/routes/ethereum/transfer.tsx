import {
  Button,
  Input,
  Option,
  Select,
  Stack,
  ToggleButtonGroup,
} from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import type { FeeMode } from '../../bindings/eth'
import { Navbar } from '../../components/navbar'
import { handle_err } from '../../lib/handle_err'
import { P, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'
import { AddressInput } from '../components'
import { EthereumTransferVM } from './transfer.vm'
import { OpenExplorerButton } from './utils/shared'

export const EthereumTransfer = observer(() => {
  const [state] = useState(() => new EthereumTransferVM())
  return (
    <Stack gap={1}>
      <Navbar />
      <P level="h3" color="primary">
        Transact on Ethereum
      </P>
      <AddressInput state={state.address} />
      <TokenSelect state={state} />
      <CurrentBalance state={state} />
      <AmountInput state={state} />
      <FeeModeSelect state={state} />
      <Button
        loading={state.is_estimating}
        disabled={state.disabled}
        sx={{ width: 'min-content' }}
        size="sm"
        onClick={() => state.estimate().catch(handle_err)}
      >
        Estimate
      </Button>
      <TransactionFee state={state} />

      {!state.tx_hash ? (
        <SendTransaction state={state} />
      ) : (
        <TransactionDetails state={state} />
      )}
    </Stack>
  )
})

const TokenSelect = observer(({ state }: { state: EthereumTransferVM }) => (
  <Select
    placeholder="Token"
    value={state.token}
    sx={{ width: 'fit-content' }}
    onChange={(_, value) => state.set_token(value ?? undefined)}
  >
    {root_store.wallet.eth.tokens_with_balance.map(t => (
      <Option key={t.address} value={t.address}>
        {t.symbol}
      </Option>
    ))}
  </Select>
))

const CurrentBalance = observer(({ state }: { state: EthereumTransferVM }) => {
  const { eth } = root_store.wallet
  const selectedToken = state.token
  if (!selectedToken) return null
  const token = eth.balance?.data?.tokens.find(
    token => token.address === selectedToken,
  )
  return (
    <P>
      Balance: {token?.balance} {token?.symbol}
    </P>
  )
})

const AmountInput = observer(({ state }: { state: EthereumTransferVM }) => (
  <Row>
    <Input
      placeholder="Amount"
      value={state.amount ?? ''}
      type="number"
      onChange={e => {
        const num = e.target.value.trim()
        if (num === '') {
          state.set_amount(undefined)
        } else {
          state.set_amount(parseFloat(num))
        }
      }}
    />
  </Row>
))

const TransactionFee = observer(({ state }: { state: EthereumTransferVM }) => {
  const { wallet } = root_store
  if (!state.estimation || !wallet.eth.usd_price) {
    return null
  }
  return (
    <P>
      Network fee: {state.estimation.fee_ceiling} gwei ~ $
      {state.estimation.fee_in_usd.toFixed(2)}
    </P>
  )
})

const SendTransaction = observer(({ state }: { state: EthereumTransferVM }) => {
  const { wallet } = root_store
  if (!state.estimation || !wallet.eth.usd_price) {
    return null
  }
  return (
    <Button
      loading={state.sending}
      onClick={() => state.execute()}
      sx={{ width: 'max-content' }}
      size="sm"
    >
      Send
    </Button>
  )
})

const TransactionDetails = observer(
  ({ state }: { state: EthereumTransferVM }) => {
    if (!state.tx_hash) {
      return null
    }
    return (
      <Stack>
        <P>
          Transaction hash <b>{state.tx_hash}</b>
        </P>
        <OpenExplorerButton path={`tx/${state.tx_hash}`} />
      </Stack>
    )
  },
)

const FeeModeSelect = observer(({ state }: { state: EthereumTransferVM }) => (
  <ToggleButtonGroup
    value={state.fee_mode}
    onChange={(_, v) => state.set_fee_mode(v)}
  >
    <Button value={'Minimal' satisfies FeeMode}>Slow</Button>
    <Button value={'Standard' satisfies FeeMode}>Standart</Button>
    <Button value={'Increased' satisfies FeeMode}>Fast</Button>
  </ToggleButtonGroup>
))
