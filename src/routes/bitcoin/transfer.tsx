import { Button, Stack, ToggleButtonGroup } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { CompactSrt } from '../../components/compact_str'
import { NumberInput } from '../../components/number_input'
import { FullScreenModal, P, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'
import { AddressInput } from '../components'
import { UtxoListModal } from './list_utxo'
import { UtxoSelectionMethodName } from './transfer.vm'

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
        <Stack>
          <P>Inputs</P>
          <Stack>
            {btc.utxo_list.selected_utxo.map(each => (
              <Row key={each.utxo_id.tx_id}>
                <Row flexWrap={'nowrap'}>
                  <CompactSrt val={each.utxo_id.tx_id} /> {each.utxo_id.vout}
                </Row>
                <P>{each.value} sat</P>
              </Row>
            ))}
          </Stack>
        </Stack>
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
