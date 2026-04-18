import {
  Button,
  Input,
  Option,
  Select,
  Stack,
  ToggleButtonGroup,
} from '@mui/joy'
import { observer } from 'mobx-react-lite'
import type { FeeMode } from '../../../bindings/eth'
import { AddressInput } from '../../../components/address_input'
import { handle_err } from '../../../lib/handle_err'
import { FullScreenModal, P, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { OpenExplorerButton } from '../utils/shared'

export const TransferModal = observer(() => {
  const state = root_store.wallet.eth.transfer
  return (
    <FullScreenModal open={state.is_open} onClose={() => state.set_open(false)}>
      <Transfer />
    </FullScreenModal>
  )
})

const Transfer = observer(() => {
  const state = root_store.wallet.eth.transfer
  return (
    <Stack gap={1}>
      <P level="h3" color="primary">
        Transact on Ethereum
      </P>
      <AddressInput state={state.address} />
      <TokenSelect />
      <CurrentBalance />
      <AmountInput />
      <FeeModeSelect />
      <Button
        loading={state.is_estimating}
        disabled={state.disabled}
        onClick={() => state.estimate().catch(handle_err)}
      >
        Estimate
      </Button>
      <TransactionFee />

      {!state.tx_hash ? <SendTransaction /> : <TransactionDetails />}
    </Stack>
  )
})

const TokenSelect = observer(() => {
  const state = root_store.wallet.eth.transfer
  return (
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
  )
})

const CurrentBalance = observer(() => {
  const { eth } = root_store.wallet
  const state = root_store.wallet.eth.transfer

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

const AmountInput = observer(() => {
  const state = root_store.wallet.eth.transfer
  return (
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
  )
})

const TransactionFee = observer(() => {
  const { wallet } = root_store
  const state = root_store.wallet.eth.transfer

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

const SendTransaction = observer(() => {
  const { wallet } = root_store
  const state = root_store.wallet.eth.transfer

  if (!state.estimation || !wallet.eth.usd_price) {
    return null
  }
  return (
    <Button loading={state.sending} onClick={() => state.execute()}>
      Send
    </Button>
  )
})

const TransactionDetails = observer(() => {
  const state = root_store.wallet.eth.transfer

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
})

const FeeModeSelect = observer(() => {
  const state = root_store.wallet.eth.transfer
  return (
    <ToggleButtonGroup
      value={state.fee_mode}
      onChange={(_, v) => state.set_fee_mode(v)}
    >
      <Button value={'Minimal' satisfies FeeMode}>Slow</Button>
      <Button value={'Standard' satisfies FeeMode}>Standart</Button>
      <Button value={'Increased' satisfies FeeMode}>Fast</Button>
    </ToggleButtonGroup>
  )
})
