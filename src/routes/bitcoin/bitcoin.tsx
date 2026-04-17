import { Button, Card, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { AccountSelector } from '../../components/account_selector'
import { Navbar } from '../../components/navbar'
import { useKeyboardRefetch } from '../../components/use_keyboard_refetch'
import { route } from '../../routes'
import { P, Progress, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'
import { ChildAddressesModal } from './list_childs'
import { UtxoListModal } from './list_utxo'
import { TransferModal } from './transfer'
import { display_sat, fmt_usd, sat2usd } from './utils/amount_formatters'

// const explorer_url = 'https://mempool.space/address/'

const Bitcoin = () => {
  const navigate = useNavigate()
  const { btc } = root_store.wallet
  const addr = btc.address

  useEffect(() => {
    if (!addr) navigate(route.unlock_wallet)
    btc.load_account_info()
  }, [addr, navigate])

  useKeyboardRefetch(() => btc.load_account_info())

  const loading =
    btc.loader.loading || btc.account_selector.account_loader.loading

  return (
    <Stack gap={1}>
      <Navbar />
      <Row gap={3}>
        <P level="h3" color="primary">
          Bitcoin
        </P>
        <AccountSelector vm={btc.account_selector} />
      </Row>
      {loading && <Progress size="sm" />}
      {addr && (
        <Card size="sm" variant="soft">
          <Stack>
            <P fontWeight="bold">{addr}</P>
          </Stack>
          <Row>
            <Button
              size="sm"
              variant="soft"
              sx={{ width: 'fit-content' }}
              onClick={() => btc.child_list.set_open(true)}
            >
              Child addresses
            </Button>
            <ChildAddressesModal />

            <Button
              size="sm"
              variant="soft"
              sx={{ width: 'fit-content' }}
              onClick={() => btc.utxo_list.open()}
            >
              Utxo
            </Button>
            <UtxoListModal store={btc.utxo_list} />

            <Button
              size="sm"
              variant="soft"
              sx={{ width: 'fit-content' }}
              onClick={() => btc.transfer.set_open(true)}
            >
              Send
            </Button>
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
      <P>Price {fmt_usd(root_store.wallet.btc.usd_price)}</P>
    </Stack>
  )
}

export default observer(Bitcoin)
