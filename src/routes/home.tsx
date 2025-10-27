import { Alert, Button, Card, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { route } from '../routes'
import { P } from '../shortcuts'
import { root_store } from './store/root'

const EthereumWalletInfo = observer(() => {
  return (
    <Card>
      <Stack>
        <P level="h3" color="primary">
          Ethereum
        </P>
        {root_store.wallet.eth && (
          <>
            <P>Address: </P>
            <P fontWeight="bold">
              {root_store.wallet.eth.address?.toLowerCase()}
            </P>
            <Stack py={2}>
              <P>Chain</P>
              <P>
                Block Height: {root_store.wallet.eth.chainInfo?.block_number}
              </P>
              <P>Block Hash: {root_store.wallet.eth.chainInfo?.block_hash}</P>
              <P>
                Base Fee Per Gas:{' '}
                {root_store.wallet.eth.chainInfo?.base_fee_per_gas}
              </P>
            </Stack>
          </>
        )}
      </Stack>
    </Card>
  )
})

const Home = () => {
  const navigate = useNavigate()

  useEffect(() => {
    if (!root_store.wallet.initialized) {
      navigate(route.unlock_wallet)
    }
  }, [])

  return (
    <Stack spacing={2}>
      {root_store.wallet.eth.err && (
        <Alert color="danger">{root_store.wallet.eth.err}</Alert>
      )}
      <EthereumWalletInfo />
      <Button
        sx={{ width: 'fit-content' }}
        size="sm"
        color="danger"
        variant="soft"
        onClick={async () => {
          await root_store.wallet.forget()
          navigate(route.unlock_wallet)
        }}
      >
        Forget wallet
      </Button>
    </Stack>
  )
}

export default observer(Home)
