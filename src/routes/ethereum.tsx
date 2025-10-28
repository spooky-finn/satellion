import { Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Address } from '../components/address'
import { Navbar } from '../components/navbar'
import { P } from '../shortcuts'
import { root_store } from '../stores/root'

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
