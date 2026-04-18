import { Button, Card, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { Navbar } from '../../../components/navbar'
import { route } from '../../../routes'
import { P, Row } from '../../../shortcuts'
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

  return (
    <Stack gap={1}>
      <Navbar />
      <P level="h3" color="primary">
        Ethereum
      </P>
      {eth && (
        <>
          {addr && (
            <Card size="sm" variant="soft">
              <P fontWeight="bold"> {addr}</P>
              <Row>
                <OpenExplorerButton path={`address/${addr}`} />
                <Button
                  onClick={() => {
                    eth.transfer.set_open(true)
                  }}
                  sx={{ width: 'min-content' }}
                >
                  Send
                </Button>
                <TransferModal />
              </Row>
            </Card>
          )}

          <BalanceCard />

          <Stack py={2}>
            <P>Ether price ${eth.usd_price}</P>
            <P>Block Height: {eth.chainInfo?.block_number}</P>
            <P>Block Hash: {eth.chainInfo?.block_hash}</P>
          </Stack>
        </>
      )}
    </Stack>
  )
})
