import { Card, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Suspense, use, useEffect } from 'react'
import { useNavigate } from 'react-router'
import { Copyable } from '../../../components/copyable'
import { useKeyboardRefetch } from '../../../components/use_keyboard_refetch'
import { ErrorBoundary } from '../../../lib/error_boundary'
import { route } from '../../../lib/routes'
import {
  B,
  NavbarAccountSelectorChainTip,
  P,
  Progress,
  Row,
} from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
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
    btc.load_chain_tip()
  })

  return (
    <Stack gap={1}>
      <NavbarAccountSelectorChainTip
        account_selector={btc.account_selector}
        network_status={{
          height: btc.chain_tip?.number,
          hash: btc.chain_tip?.hash,
          price: btc.usd_price,
        }}
      />
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
          <Copyable copy val={btc.address} sx={{ fontWeight: 600 }} />
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

            <B
              sx={{ ml: 'auto' }}
              loading={btc.discovery_loader.loading}
              variant="plain"
              onClick={() => btc.discover_wallet()}
            >
              Discover
            </B>
            <TransferModal />
          </Row>
          {btc.discovery_loader.data && (
            <P level="body-xs">
              Discovery found {btc.discovery_loader.data.accounts.length}{' '}
              accounts, added {btc.discovery_loader.data.paths_added} paths and{' '}
              {btc.discovery_loader.data.utxos_added} UTXOs
            </P>
          )}
        </Card>
      )}
      <DisplaySat satoshis={btc.total_balance_sat} usd_price={btc.usd_price} />
      <PendingTxsSection />
      <FeeBumpModal />
    </Stack>
  )
})

export default BitcoinWallet
