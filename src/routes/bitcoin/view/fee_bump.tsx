import { Divider, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { CompactSrt } from '../../../components/compact_str'
import { NumberInput } from '../../../components/number_input'
import { B, FullScreenModal, P, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { DisplaySat } from '../utils/display_sat'
import { ExplorerLink } from '../utils/explorer'
import { FeeBumpState } from '../view_model/fee_bump.vm'

export const FeeBumpModal = observer(() => {
  const { fee_bump } = root_store.wallet.btc
  return (
    <FullScreenModal
      open={!!fee_bump.parent_tx_id}
      onClose={() => fee_bump.close()}
    >
      <P level="h3" color="primary">
        Bump fee (CPFP)
      </P>
      <Stack gap={2}>
        <FeeBumpBody />
      </Stack>
    </FullScreenModal>
  )
})

const FeeBumpBody = observer(() => {
  const { fee_bump } = root_store.wallet.btc
  switch (fee_bump.state) {
    case FeeBumpState.Result:
      return <ResultView />
    default:
      return <InputView />
  }
})

const InputView = observer(() => {
  const { btc } = root_store.wallet
  const { fee_bump } = btc
  const parent_id = fee_bump.parent_tx_id ?? ''

  return (
    <>
      <Stack gap={0.5}>
        <P level="body-sm">Parent transaction</P>
        <CompactSrt
          copy
          val={parent_id}
          fontFamily={'monospace'}
          level="body-xs"
        />
      </Stack>

      <P level="body-xs" color="neutral">
        Spends every output of the parent that belongs to this wallet into a new
        self-send. Miners with package-aware mempool policies will pull the
        parent in once the child pays enough.
      </P>

      <Row alignItems={'center'}>
        <NumberInput
          placeholder="Fee rate"
          value={fee_bump.fee_rate_sat_vb}
          onChange={v => fee_bump.set_fee_rate(v ?? 0)}
          width={120}
          endDecorator={<P>sat/vB</P>}
        />
      </Row>

      {fee_bump.error && <P color="danger">{fee_bump.error}</P>}

      <Row>
        <B variant="plain" onClick={() => fee_bump.close()}>
          Cancel
        </B>
        <B
          loading={fee_bump.state === FeeBumpState.Sending}
          onClick={() => fee_bump.submit()}
        >
          Broadcast CPFP
        </B>
      </Row>
    </>
  )
})

const ResultView = observer(() => {
  const { btc } = root_store.wallet
  const { fee_bump } = btc
  const child = fee_bump.result
  if (!child) return null
  return (
    <>
      <P>Child transaction broadcast</P>
      <Stack gap={0.5}>
        <P level="body-sm">Child txid</P>
        <CompactSrt
          copy
          val={child.child_tx_id}
          fontFamily={'monospace'}
          level="body-xs"
        />
      </Stack>
      <Stack gap={0.5}>
        <P level="body-sm">Child fee</P>
        <DisplaySat
          label=""
          satoshis={child.child_fee}
          usd_price={btc.usd_price}
          fraction_digits={2}
        />
      </Stack>
      <Divider />
      <Row>
        <ExplorerLink type="tx" txid={child.child_tx_id} />
        <B variant="plain" onClick={() => fee_bump.close()}>
          Close
        </B>
      </Row>
    </>
  )
})
