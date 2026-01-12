import { Card, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { Navbar } from '../../components/navbar'
import { route } from '../../routes'
import { P, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'
import { DeriveChildAddress } from './derive_child'
import { ListDerivedAddresses } from './list_childs'
import { ListUtxo } from './list_utxo'

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
      <P level="h3" color="primary">
        Bitcoin
      </P>
      {addr && (
        <Card size="sm" variant="soft">
          <Row gap={1}>
            <P>Main Address</P>
            <P fontWeight="bold"> {addr}</P>
          </Row>
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
          <P>
            Blockchain sync {btc.status} - height {btc.height},{' '}
            {btc.progress.toFixed(0)}%
          </P>
          {btc.warning && (
            <P level="body-xs" color="warning">
              {btc.warning}
            </P>
          )}
          <P>Price {root_store.wallet.btc.usd_price} USD</P>
        </>
      )}
    </Stack>
  )
}

export default observer(Bitcoin)
