import { Divider, Stack, ToggleButtonGroup } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { AddressInput } from '../../../components/address_input'
import { CompactSrt } from '../../../components/compact_str'
import { NumberInput } from '../../../components/number_input'
import { B, FullScreenModal, P, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { sat2usd } from '../utils/amount_formatters'
import {
  TransferState,
  UtxoSelectionMethodKind,
} from '../view_model/transfer.vm'
import { UtxoListModal } from './list_utxo'

export const TransferModal = observer(() => {
  const { transfer } = root_store.wallet.btc
  return (
    <FullScreenModal
      open={transfer.is_open}
      onClose={() => transfer.set_open(false)}
    >
      <P level="h3">Send bitcoin</P>
      {transfer.state === TransferState.Result ? (
        <TransferResult />
      ) : (
        <TransferForm />
      )}
    </FullScreenModal>
  )
})

const TransferForm = observer(() => {
  const { btc } = root_store.wallet
  const { transfer } = btc
  return (
    <Stack gap={1}>
      <AddressInput state={transfer.address} />
      <Stack>
        <P level="body-sm">Utxo selection method</P>
        <UtxoSelectionMethod />
        <SelectedInputsSummary />
      </Stack>
      <Row alignItems={'center'}>
        <NumberInput
          placeholder="Transfer amount"
          value={transfer.transfer_amount}
          onChange={v => {
            transfer.set_transfer_amount(v)
          }}
          width={300}
          endDecorator={<P>SAT</P>}
        />
        <P>{transfer.estimated_transfer_value(btc.usd_price)}</P>
      </Row>
      {transfer.error && <P color="danger">{transfer.error}</P>}
      {transfer.state === TransferState.Estimate && (
        <B onClick={() => transfer.estimate(btc.utxo_list.selected_utxo)}>
          Estimate
        </B>
      )}
      {transfer.state === TransferState.Sending && (
        <B onClick={() => transfer.execute()}>Send</B>
      )}
    </Stack>
  )
})

const TransferResult = () => {
  const { btc } = root_store.wallet
  const { transfer } = btc
  return (
    <Stack>
      <P>Transaction sent</P>
      <P fontFamily={'monospace'}>{transfer.broadcast_result?.tx_id}</P>
      <B onClick={() => transfer.reset()} variant="plain">
        Send another
      </B>
    </Stack>
  )
}

const SelectedInputsSummary = observer(() => {
  const { utxo_list } = root_store.wallet.btc

  if (!utxo_list.selected_utxo.length) return
  return (
    <Stack>
      <P>Inputs</P>
      <Stack>
        {utxo_list.selected_utxo.map(each => (
          <Row key={each.utxo_id.tx_id}>
            <Row flexWrap={'nowrap'}>
              <CompactSrt
                level="body-xs"
                val={`${each.utxo_id.tx_id}_${each.utxo_id.vout}`}
              />
            </Row>
            <P>{each.value} sat</P>
          </Row>
        ))}
        <Divider />
        <P>Total</P>
        <P>
          {utxo_list.selected_utxo_total_value} sat ~
          {sat2usd(
            utxo_list.selected_utxo_total_value.toString(),
            root_store.wallet.btc.usd_price,
          )}{' '}
        </P>
      </Stack>
    </Stack>
  )
})

const UtxoSelectionMethod = observer(() => {
  const { btc } = root_store.wallet
  const { transfer } = root_store.wallet.btc
  return (
    <Row>
      <ToggleButtonGroup
        variant="soft"
        value={transfer.utxo_selection_method}
        onChange={(_, v) => transfer.set_utxo_selection_method(v)}
      >
        <B value={UtxoSelectionMethodKind.Auto}>Auto</B>
        <B value={UtxoSelectionMethodKind.Manual}>Manual</B>
      </ToggleButtonGroup>
      {transfer.show_utxo_select_button && (
        <>
          <B onClick={() => btc.utxo_list.open(true)}>Select</B>
          <UtxoListModal />
        </>
      )}
    </Row>
  )
})
