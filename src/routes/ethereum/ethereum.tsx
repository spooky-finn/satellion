import { Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Address } from '../../components/address'
import { Navbar } from '../../components/navbar'
import { route } from '../../routes'
import { LinkButton, P } from '../../shortcuts'
import { root_store } from '../../stores/root'
import { BalanceCard } from './balances'

const explorer_url = 'https://etherscan.io/address/'

const Ethereum = () => {
  return (
    <Stack gap={1}>
      <Navbar />
      <P level="h3" color="primary">
        Ethereum
      </P>
      {root_store.wallet.eth && (
        <>
          <Address
            addr={root_store.wallet.eth.address}
            explorer_url={explorer_url + root_store.wallet.eth.address}
          />
          <BalanceCard />
          <LinkButton to={route.ethereum_send} sx={{ width: 'min-content' }}>
            Send
          </LinkButton>
          <Stack py={2}>
            <P>ETH price {root_store.wallet.eth.balance?.eth_price}</P>
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
