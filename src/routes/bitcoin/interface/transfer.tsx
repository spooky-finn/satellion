import { Button, Divider, Stack, ToggleButtonGroup } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { AddressInput } from '../../../components/address_input'
import { CompactSrt } from '../../../components/compact_str'
import { NumberInput } from '../../../components/number_input'
import { FullScreenModal, P, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { UtxoSelectionMethodName } from '../view_model/transfer.vm'
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
  const state = btc.transfer
  return (
    <Stack gap={1}>
      <P level="h3">Send bitcoin</P>
      <AddressInput state={state.address} />
      <Stack>
        <P level="body-sm">Utxo selection method</P>
        <UtxoSelectionMethod />
        <SelectedInputsSummary />
      </Stack>
      <Row alignItems={'center'}>
        <NumberInput
          placeholder="Transfer amount"
          value={state.transfer_amount}
          onChange={v => {
            state.set_transfer_amount(v)
          }}
          width={300}
          endDecorator={<P>SAT</P>}
        />
        <P>{state.estimated_transfer_value(btc.usd_price)}</P>
      </Row>
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
        <P>{utxo_list.selected_utxo_total_value} sat ~ </P>
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
          <Button
            onClick={() => btc.utxo_list.open(true)}
            sx={{ width: 'fit-content' }}
          >
            Select
          </Button>
          <UtxoListModal store={btc.utxo_list} />
        </>
      )}
    </Row>
  )
})
