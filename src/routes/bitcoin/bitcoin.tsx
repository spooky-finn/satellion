import { Card, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { AccountSelector } from '../../components/account_selector'
import { Navbar } from '../../components/navbar'
import { route } from '../../routes'
import { P, Progress, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'
import { DeriveChildAddress } from './derive_child'
import { ListDerivedAddresses } from './list_childs'
import { ListUtxo } from './list_utxo'
import { display_sat, fmt_usd, sat2usd } from './utils/amount_formatters'

const explorer_url = 'https://mempool.space/address/'

const Bitcoin = () => {
  const navigate = useNavigate()
  const { btc } = root_store.wallet
  const addr = btc.address

  useEffect(() => {
    if (!addr) navigate(route.unlock_wallet)
  }, [addr, navigate])

  return (
    <Stack gap={1}>
      <Navbar />
      <Row gap={3}>
        <P level="h3" color="primary">
          Bitcoin
        </P>
        <AccountSelector vm={btc.account_selector} />
      </Row>
      {btc.account_selector.account_loader.loading && <Progress size="sm" />}
      {addr && (
        <Card size="sm" variant="soft">
          <Stack>
            <P fontWeight="bold">{addr}</P>
            <P level="body-xs">Main Address</P>
          </Stack>
          <P level="body-xs">
            Do not share this address with untrusted parties who may send
            tainted or illicit coins.
            <br />
            Receiving funds from suspicious sources can link your wallet to
            illegal activity.
            <br />
            For secure acceptance of funds, consider generating dedicated child
            address per transaction.
          </P>
          <Row>
            <DeriveChildAddress />
            <ListDerivedAddresses />
            <ListUtxo />
          </Row>
        </Card>
      )}
      {btc.height && (
        <>
          <P level="body-xs">Blockchain head at {btc.height}</P>
        </>
      )}
      {btc.warning && (
        <P level="body-xs" color="warning">
          {btc.warning}
        </P>
      )}
      <P>
        Balance {display_sat(btc.total_balance_sat)} ~{' '}
        <P level="body-xs">{sat2usd(btc.total_balance_sat, btc.usd_price)}</P>
      </P>
      <P>Price {fmt_usd(root_store.wallet.btc.usd_price)}</P>
    </Stack>
  )
}

export default observer(Bitcoin)
