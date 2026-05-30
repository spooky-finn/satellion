import { Chip, Stack, Table } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Suspense, use } from 'react'
import { CompactSrt } from '../../../components/compact_str'
import { ErrorBoundary } from '../../../lib/error_boundary'
import { FullScreenModal, P, Progress, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { display_sat, sat2usd } from '../utils/amount_formatters'
import { DisplaySat } from '../utils/display_sat'

export const UtxoListModal = observer(() => {
  const { utxo_list } = root_store.wallet.btc
  return (
    <FullScreenModal open={utxo_list.is_open} onClose={() => utxo_list.close()}>
      <P level="h3">Unspent transaction outputs</P>
      <ErrorBoundary>
        <Suspense fallback={<Progress />}>
          <UtxoList />
        </Suspense>
      </ErrorBoundary>
    </FullScreenModal>
  )
})

const UtxoList = observer(() => {
  const { btc } = root_store.wallet
  const { utxo_list } = root_store.wallet.btc
  use(utxo_list.sync.promise)
  return (
    <Stack sx={{ overflow: 'auto', mt: 0 }}>
      {utxo_list.utxo.length === 0 ? (
        <P>No utxos yet.</P>
      ) : (
        <>
          <P>In total {utxo_list.utxo.length} utxo</P>
          <DisplaySat
            satoshis={utxo_list.total_value_sat}
            usd_price={btc.usd_price}
          />
          <Table variant="plain" stickyHeader size="sm">
            <TableHead />
            <TableBody />
          </Table>
        </>
      )}
    </Stack>
  )
})

const TableHead = () => (
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
)

const TableBody = observer(() => {
  const { utxo_list } = root_store.wallet.btc
  return (
    <tbody>
      {utxo_list.utxo.map((utxo, index) => {
        const key = utxo.utxo_id.tx_id + utxo.utxo_id.vout
        return (
          <tr
            style={{
              cursor: utxo_list.selection_mode ? 'pointer' : 'auto',
            }}
            key={key}
            onClick={() =>
              utxo_list.selection_mode && utxo_list.select_utxo(index)
            }
          >
            <td>
              <Row>
                {utxo_list._selected_utxo.includes(index) && (
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
              <P sx={{ fontFamily: 'monospace' }}>{display_sat(utxo.value)}</P>
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
  )
})
