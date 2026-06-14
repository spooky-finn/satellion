import { Card, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Suspense, use, useEffect } from 'react'
import { useNavigate } from 'react-router'
import { AccountSelector } from '../../../components/account_selector'
import { CompactSrt } from '../../../components/compact_str'
import { Navbar } from '../../../components/navbar'
import { useKeyboardRefetch } from '../../../components/use_keyboard_refetch'
import { ErrorBoundary } from '../../../lib/error_boundary'
import { route } from '../../../lib/routes'
import { B, P, Progress, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { fmt_usd } from '../utils/amount_formatters'
import { DisplaySat } from '../utils/display_sat'
import { FeeBumpModal } from './fee_bump'
import { ChildAddressesModal } from './list_childs'
import { UtxoListModal } from './list_utxo'
import { PendingTxsSection } from './pending_txs'
import { TransferModal } from './transfer'

const BitcoinWallet = observer(() => {
  const navigate = useNavigate()
  const { btc } = root_store.wallet
  const addr = btc.address

  useEffect(() => {
    if (!addr) navigate(route.unlock_wallet)
  }, [addr, navigate])

  useKeyboardRefetch(async () => {
    btc.account_info.refresh()
  })

  return (
    <Stack gap={1}>
      <Navbar />
      <Row gap={3}>
        <AccountSelector vm={btc.account_selector} />
      </Row>
      <ErrorBoundary>
        <Suspense fallback={<Progress size="sm" />}>
          <BitcoinDetails />
        </Suspense>
      </ErrorBoundary>
    </Stack>
  )
})

const BitcoinDetails = observer(() => {
  const { btc } = root_store.wallet
  use(btc.account_info.promise)
  return (
    <Stack gap={1}>
      {btc.address && (
        <Card size="sm" variant="soft">
          <CompactSrt copy val={btc.address} sx={{ fontWeight: 600 }} />
          <Row>
            <B variant="soft" onClick={() => btc.child_list.set_open(true)}>
              Child addresses
            </B>
            <ChildAddressesModal />

            <B variant="soft" onClick={() => btc.utxo_list.open()}>
              Utxo
            </B>
            <UtxoListModal />

            <B onClick={() => btc.transfer.set_open(true)}>Send</B>
            <TransferModal />
          </Row>
        </Card>
      )}
      {btc.height && <P level="body-xs">Blockchain head at {btc.height}</P>}
      <DisplaySat satoshis={btc.total_balance_sat} usd_price={btc.usd_price} />
      <P>Price {fmt_usd(btc.usd_price)}</P>
      <PendingTxsSection />
      <FeeBumpModal />
    </Stack>
  )
})

export default BitcoinWallet
