import { Card, Divider, Stack, ToggleButtonGroup } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { AddressInput } from '../../../components/address_input'
import { CompactSrt } from '../../../components/compact_str'
import { NumberInput } from '../../../components/number_input'
import { B, P, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { DisplaySat } from '../utils/display_sat'
import { UtxoSelectionMethodKind } from '../view_model/transfer.vm'
import { UtxoListModal } from './list_utxo'

export const CreateTransfer = observer(() => {
  const { btc } = root_store.wallet
  const { transfer } = btc
  return (
    <>
      <AddressInput state={transfer.address} />
      <Stack gap={1}>
        <P level="body-sm">Utxo selection method</P>
        <UtxoSelectionMethod />
        <SelectedInputsSummary />
      </Stack>
      <Row alignItems={'center'}>
        <NumberInput
          placeholder="Amount"
          value={transfer.transfer_amount}
          onChange={v => {
            transfer.set_transfer_amount(v)
          }}
          width={150}
          endDecorator={<P>SAT</P>}
        />
        <P>{transfer.estimated_transfer_value(btc.usd_price)}</P>
      </Row>
      {transfer.error && <P color="danger">{transfer.error}</P>}
      <B onClick={() => transfer.estimate(btc.utxo_list.selected_utxo)}>
        Estimate
      </B>
    </>
  )
})

const SelectedInputsSummary = observer(() => {
  const { btc } = root_store.wallet
  const { utxo_list } = btc

  if (!utxo_list.selected_utxo.length) return
  return (
    <Card variant="outlined" color="neutral">
      <Stack gap={1}>
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
          <Divider sx={{ my: 0.5 }} />
          <DisplaySat
            usd_price={btc.usd_price}
            satoshis={utxo_list.selected_utxo_total_value}
            label="In total"
          />
        </Stack>
      </Stack>
    </Card>
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
        onChange={(_, v) => {
          transfer.set_utxo_selection_method(v)
          if (v === UtxoSelectionMethodKind.Manual) btc.utxo_list.open(true)
        }}
      >
        <B value={UtxoSelectionMethodKind.Auto}>Auto</B>
        <B value={UtxoSelectionMethodKind.Manual}>Manual</B>
      </ToggleButtonGroup>
      {transfer.show_utxo_select_button && <UtxoListModal />}
    </Row>
  )
})
