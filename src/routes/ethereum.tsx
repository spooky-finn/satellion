import { IconButton, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Address } from '../components/address'
import { RefreshIcon } from '../components/icons/refresh.icon'
import { Navbar } from '../components/navbar'
import { P, Row } from '../shortcuts'
import { root_store } from '../stores/root'

const Balance = observer(() => (
  <Row alignItems={'center'}>
    <P>Balance: {root_store.wallet.eth.balance} ETH</P>
    <IconButton onClick={() => root_store.wallet.eth.getBalance()}>
      <RefreshIcon />
    </IconButton>
  </Row>
))

const Ethereum = () => {
  return (
    <Stack gap={1}>
      <Navbar />
      <P level="h3" color="primary">
        Ethereum
      </P>
      {root_store.wallet.eth && (
        <>
          <Address addr={root_store.wallet.eth.address} />
          <Balance />
          <Stack py={2}>
            <P>Chain</P>
            <P>Block Height: {root_store.wallet.eth.chainInfo?.block_number}</P>
            <P>Block Hash: {root_store.wallet.eth.chainInfo?.block_hash}</P>
            <P>
              Base Fee Per Gas:{' '}
              {root_store.wallet.eth.chainInfo?.base_fee_per_gas}
            </P>
          </Stack>
        </>
      )}
    </Stack>
  )
}

export default observer(Ethereum)
