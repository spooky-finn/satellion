import { Chip, Stack, Table } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { CompactSrt } from '../../../components/compact_str'
import { FullScreenModal, P, Progress, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { display_sat, sat2usd } from '../utils/amount_formatters'
import type { UtxoListVM } from '../view_model/utxo_list.vm'

export const UtxoListModal = observer(({ store }: { store: UtxoListVM }) => (
  <FullScreenModal open={store.is_open} onClose={() => store.close()}>
    <UtxoList store={store} />
  </FullScreenModal>
))

const UtxoList = observer(({ store }: { store: UtxoListVM }) => (
  <>
    <P level="h3">Unspent transaction outputs</P>
    {store.loader.loading && <Progress />}
    <Stack sx={{ overflow: 'auto', mt: 0 }}>
      {store.utxo.length === 0 ? (
        <P>No utxos yet.</P>
      ) : (
        <>
          <P>
            In total {store.utxo.length} utxo contains{' '}
            {display_sat(store.total_value_sat)}
          </P>

          <Table variant="plain" stickyHeader size="sm">
            <thead>
              <tr>
                <th>
                  <P>Derivation path</P>
                </th>
                <th>
                  <P>Label</P>
                </th>
                <th>
                  <P>Transaction ID</P>
                </th>
                <th>
                  <P>Amount</P>
                </th>
                <th>
                  <P>Value</P>
                </th>
              </tr>
            </thead>

            <tbody>
              {store.utxo.map((utxo, index) => {
                const key = utxo.utxo_id.tx_id + utxo.utxo_id.vout
                return (
                  <tr
                    style={{
                      cursor: store.selection_mode ? 'pointer' : 'auto',
                    }}
                    key={key}
                    onClick={() =>
                      store.selection_mode && store.select_utxo(index)
                    }
                  >
                    <td>
                      <Row>
                        {store._selected_utxo.includes(index) && (
                          <Chip color="danger" size="sm" variant="solid" />
                        )}
                        <P fontFamily={'monospace'} level="body-xs">
                          {utxo.deriv_path}
                        </P>
                      </Row>
                    </td>
                    <td>
                      <P>{utxo.address_label}</P>
                    </td>
                    <td>
                      <CompactSrt copy val={utxo.utxo_id.tx_id} />
                    </td>
                    <td>
                      <P sx={{ fontFamily: 'monospace' }}>
                        {display_sat(utxo.value)}
                      </P>
                    </td>
                    <td>
                      <P sx={{ fontFamily: 'monospace' }}>
                        {sat2usd(utxo.value, root_store.wallet.btc.usd_price)}
                      </P>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </Table>
        </>
      )}
    </Stack>
  </>
))
