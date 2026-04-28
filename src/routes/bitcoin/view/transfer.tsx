import { Button, Divider, Stack, ToggleButtonGroup } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { AddressInput } from '../../../components/address_input'
import { CompactSrt } from '../../../components/compact_str'
import { NumberInput } from '../../../components/number_input'
import { FullScreenModal, P, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { sat2usd } from '../utils/amount_formatters'
import {
  TransferState,
  UtxoSelectionMethodName,
} from '../view_model/transfer.vm'
import { UtxoListModal } from './list_utxo'

export const TransferModal = observer(() => {
  const { transfer } = root_store.wallet.btc
  return (
    <FullScreenModal
      open={transfer.is_open}
      onClose={() => transfer.set_open(false)}
    >
      <TransferForm />
    </FullScreenModal>
  )
})

const TransferForm = observer(() => {
  const { btc } = root_store.wallet
  const { transfer } = btc
  return (
    <Stack gap={1}>
      <P level="h3">Send bitcoin</P>
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
        <Button onClick={() => transfer.estimate(btc.utxo_list.selected_utxo)}>
          Estimate
        </Button>
      )}
      {transfer.state === TransferState.Sending && (
        <Button onClick={() => transfer.execute()}>Send</Button>
      )}
    </Stack>
  )
})

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
  const state = root_store.wallet.btc.transfer
  return (
    <Row gap={1}>
      <ToggleButtonGroup
        variant="soft"
        value={state.utxo_selection_method}
        onChange={(_, v) => state.set_utxo_selection_method(v)}
      >
        <Button value={UtxoSelectionMethodName.Auto}>Auto</Button>
        <Button value={UtxoSelectionMethodName.Manual}>Manual</Button>
      </ToggleButtonGroup>
      {state.show_utxo_select_button && (
        <>
          <Button onClick={() => btc.utxo_list.open(true)}>Select</Button>
          <UtxoListModal store={btc.utxo_list} />
        </>
      )}
    </Row>
  )
})
