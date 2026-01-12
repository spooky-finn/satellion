import { Card, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { Navbar } from '../../components/navbar'
import { route } from '../../routes'
import { LinkButton, P, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'
import { BalanceCard } from './balances'
import { OpenExplorerButton } from './utils/shared'

export const Ethereum = observer(() => {
  const navigate = useNavigate()
  const addr = root_store.wallet.eth.address

  useEffect(() => {
    if (!addr) navigate(route.unlock_wallet)
  }, [addr, navigate])

  return (
    <Stack gap={1}>
      <Navbar />
      <P level="h3" color="primary">
        Ethereum
      </P>
      {root_store.wallet.eth && (
        <>
          {addr && (
            <Card size="sm" variant="soft">
              <P fontWeight="bold"> {addr}</P>
              <Row>
                <OpenExplorerButton path={`address/${addr}`} />
                <LinkButton
                  to={route.ethereum_send}
                  sx={{ width: 'min-content' }}
                >
                  Send
                </LinkButton>
              </Row>
            </Card>
          )}

          <BalanceCard />

          <Stack py={2}>
            <P>Ether price ${root_store.wallet.eth.usd_price}</P>
            <P>Block Height: {root_store.wallet.eth.chainInfo?.block_number}</P>
            <P>Block Hash: {root_store.wallet.eth.chainInfo?.block_hash}</P>
          </Stack>
        </>
      )}
    </Stack>
  )
})
