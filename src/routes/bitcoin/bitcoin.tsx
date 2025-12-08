import { Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Address } from '../../components/address'
import { Navbar } from '../../components/navbar'
import { P } from '../../shortcuts'
import { root_store } from '../../stores/root'

const explorer_url = 'https://mempool.space/address/'

const Bitcoin = () => {
  return (
    <Stack gap={1}>
      <Navbar />
      <P level="h3" color="primary">
        Bitcoin
      </P>
      {root_store.wallet.btc.address && (
        <>
          <Address addr={root_store.wallet.btc.address} />
        </>
      )}
    </Stack>
  )
}

export default observer(Bitcoin)
