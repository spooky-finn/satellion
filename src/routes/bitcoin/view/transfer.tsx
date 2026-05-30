import { Divider, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { SendTxButton } from '../../../components/send_tx_button'
import { B, FullScreenModal, P, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { DisplaySat } from '../utils/display_sat'
import { ExplorerLink } from '../utils/explorer'
import { TransferState } from '../view_model/transfer.vm'
import { CreateTransfer } from './transfer_create'

export const TransferModal = observer(() => {
  const { transfer } = root_store.wallet.btc

  return (
    <FullScreenModal
      open={transfer.is_open}
      onClose={() => transfer.set_open(false)}
    >
      <P level="h3" color="primary">
        Send bitcoin
      </P>
      <Stack gap={2}>
        <TransferBody />
      </Stack>
    </FullScreenModal>
  )
})

const TransferBody = observer(() => {
  const { transfer } = root_store.wallet.btc
  switch (transfer.state) {
    case TransferState.Result:
      return <TransferResult />
    case TransferState.Sending:
      return <ValidateTransfer />
    default:
      return <CreateTransfer />
  }
})

const ValidateTransfer = observer(() => {
  const { btc } = root_store.wallet
  const { transfer } = btc

  const amount = transfer.transfer_amount ?? 0
  const fee = transfer.estimateion?.fee ?? 0

  return (
    <>
      <P level="title-md">Verify transfer</P>

      <Stack gap={0.5}>
        <P level="body-sm">Recipient</P>
        <P fontFamily={'monospace'} sx={{ wordBreak: 'break-all' }}>
          {transfer.address.val}
        </P>
      </Stack>

      <Stack gap={0.5}>
        <P level="body-sm">Amount</P>
        <DisplaySat
          label=""
          satoshis={amount}
          usd_price={btc.usd_price}
          fraction_digits={2}
        />
      </Stack>

      <Stack gap={0.5}>
        <P level="body-sm">Network fee</P>
        <DisplaySat
          label=""
          satoshis={fee}
          usd_price={btc.usd_price}
          fraction_digits={2}
        />
      </Stack>

      <Divider />

      <Stack gap={0.5}>
        <P level="body-sm">Total</P>
        <DisplaySat
          label=""
          satoshis={amount + fee}
          usd_price={btc.usd_price}
          fraction_digits={2}
        />
      </Stack>

      {transfer.error && <P color="danger">{transfer.error}</P>}

      <Row>
        <B variant="plain" onClick={() => transfer.back_to_estimate()}>
          Back
        </B>
        <SendTxButton onSend={() => transfer.execute()}>
          Hold to send
        </SendTxButton>
      </Row>
    </>
  )
})

const TransferResult = observer(() => {
  const { btc } = root_store.wallet
  const { transfer } = btc
  return (
    <>
      <P>Transaction sent</P>
      <P fontFamily={'monospace'}>{transfer.broadcast_result?.tx_id}</P>
      {transfer.broadcast_result?.tx_id && (
        <ExplorerLink type="tx" txid={transfer.broadcast_result.tx_id} />
      )}
      <B onClick={() => transfer.reset()} variant="plain">
        Send another
      </B>
    </>
  )
})
