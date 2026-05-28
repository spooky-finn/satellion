import { Button, Card, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Suspense, use, useEffect } from 'react'
import { useNavigate } from 'react-router'
import { AccountSelector } from '../../../components/account_selector'
import { CompactSrt } from '../../../components/compact_str'
import { Navbar } from '../../../components/navbar'
import { useKeyboardRefetch } from '../../../components/use_keyboard_refetch'
import { ErrorBoundary } from '../../../lib/error_boundary'
import { route } from '../../../lib/routes'
import { P, Progress, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { display_sat, fmt_usd, sat2usd } from '../utils/amount_formatters'
import { ChildAddressesModal } from './list_childs'
import { UtxoListModal } from './list_utxo'
import { TransferModal } from './transfer'

const BitcoinWallet = observer(() => {
  const navigate = useNavigate()
  const { btc } = root_store.wallet
  const addr = btc.address

  useEffect(() => {
    if (!addr) {
      navigate(route.unlock_wallet)
      return
    }
    btc.account_info.refresh()
  }, [addr, navigate, btc.account_info])

  useKeyboardRefetch(async () => {
    btc.account_info.refresh()
  })

  return (
    <Stack gap={1}>
      <Navbar />
      <Row gap={3}>
        <P level="h3" color="primary">
          Bitcoin
        </P>
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
          <CompactSrt copy val={btc.address} />
          <Row>
            <Button
              variant="soft"
              onClick={() => btc.child_list.set_open(true)}
            >
              Child addresses
            </Button>
            <ChildAddressesModal />

            <Button variant="soft" onClick={() => btc.utxo_list.open()}>
              Utxo
            </Button>
            <UtxoListModal />

            <Button onClick={() => btc.transfer.set_open(true)}>Send</Button>
            <TransferModal />
          </Row>
        </Card>
      )}
      {btc.height && <P level="body-xs">Blockchain head at {btc.height}</P>}
      {btc.warning && (
        <P level="body-xs" color="warning">
          {btc.warning}
        </P>
      )}
      <P>
        Balance {display_sat(btc.total_balance_sat)} ~{' '}
        <P level="body-xs">{sat2usd(btc.total_balance_sat, btc.usd_price)}</P>
      </P>
      <P>Price {fmt_usd(btc.usd_price)}</P>
    </Stack>
  )
})

export default BitcoinWallet
