import { Card, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { Copyable } from '../../../components/copyable'
import { useKeyboardRefetch } from '../../../components/use_keyboard_refetch'
import { route } from '../../../lib/routes'
import { B, NavbarAccountSelectorChainTip, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { OpenExplorerButton } from '../utils/shared'
import { BalanceCard } from './balances'
import { TransferModal } from './transfer'

export const EthereumWallet = observer(() => {
  const navigate = useNavigate()
  const { eth } = root_store.wallet
  const addr = eth.address

  useEffect(() => {
    if (!addr) navigate(route.unlock_wallet)
  }, [addr, navigate])

  useKeyboardRefetch(async () => {
    eth.load_active_account()
  })

  return (
    <Stack gap={1}>
      <NavbarAccountSelectorChainTip
        account_selector={eth.account_selector}
        network_status={{
          height: eth.chainInfo?.block_number,
          hash: eth.chainInfo?.block_hash,
          price: eth.usd_price,
        }}
      />
      {eth && (
        <>
          {addr && (
            <Card size="sm" variant="soft">
              <Copyable sx={{ fontWeight: 600 }} val={addr} copy />
              <Row>
                <OpenExplorerButton path={`address/${addr}`} />
                <B
                  onClick={() => {
                    eth.transfer.set_open(true)
                  }}
                >
                  Send
                </B>
                <TransferModal />
              </Row>
            </Card>
          )}

          <BalanceCard />
        </>
      )}
    </Stack>
  )
})
