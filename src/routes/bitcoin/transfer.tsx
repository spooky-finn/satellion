import { Button, Stack, ToggleButtonGroup } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Navbar } from '../../components/navbar'
import { NumberInput } from '../../components/number_input'
import { P, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'
import { AddressInput } from '../components'
import { ListUtxo } from './list_utxo'
import { UtxoSelectionMethodName } from './transfer.vm'

export const BitcoinTransfer = observer(() => {
  const { btc } = root_store.wallet
  const state = btc.transfer
  return (
    <Stack gap={1}>
      <Navbar />
      <P level="h3" color="primary">
        Send bitcoin
      </P>
      <AddressInput state={state.address} />
      <Stack>
        <P level="body-sm">Utxo selection method</P>
        <UtxoSelectionMethod />
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
            onClick={() => state.utxo_select_moda.open(true)}
            sx={{ width: 'fit-content' }}
          >
            Select
          </Button>
          <ListUtxo store={state.utxo_select_moda} />
        </>
      )}
    </Row>
  )
})
