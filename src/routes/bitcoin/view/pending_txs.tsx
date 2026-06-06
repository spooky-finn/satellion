import { Card, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { CompactSrt } from '../../../components/compact_str'
import { B, P, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { DisplaySat } from '../utils/display_sat'

export const PendingTxsSection = observer(() => {
  const { btc } = root_store.wallet
  const pending = btc.utxo_list.pending_parent_txs
  if (pending.length === 0) return null

  return (
    <Card size="sm" variant="soft" color="warning">
      <P level="title-sm">Pending transactions</P>
      <P level="body-xs" color="neutral">
        Outputs from these txs are still in the mempool. Bump the fee with a
        CPFP child to help them confirm.
      </P>
      <Stack gap={1}>
        {pending.map(p => (
          <Row
            key={p.parent_tx_id}
            justifyContent={'space-between'}
            alignItems={'center'}
          >
            <Stack gap={0.25}>
              <CompactSrt
                copy
                val={p.parent_tx_id}
                fontFamily={'monospace'}
                level="body-xs"
              />
              <DisplaySat
                label="Pending"
                satoshis={p.value_sat}
                usd_price={btc.usd_price}
                fraction_digits={2}
              />
            </Stack>
            <B
              size="sm"
              variant="solid"
              onClick={() => btc.fee_bump.open(p.parent_tx_id)}
            >
              Bump fee
            </B>
          </Row>
        ))}
      </Stack>
    </Card>
  )
})
