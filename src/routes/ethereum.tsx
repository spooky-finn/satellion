import { Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Navbar } from '../components/navbar'
import { P } from '../shortcuts'
import { root_store } from '../stores/root'

const Ethereum = () => {
  return (
    <Stack>
      <Navbar />
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
