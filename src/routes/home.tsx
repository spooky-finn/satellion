import { Box } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { route } from '../routes'
import { P } from '../shortcuts'
import { root_store } from './store/root'

const Home = () => {
  const navigate = useNavigate()

  useEffect(() => {
    if (!root_store.wallet.initialized) {
      navigate(route.unlock_wallet)
    }
  }, [])

  return (
    <Box>
      <P level="h2" color="primary">
        Ethereum Wallet {root_store.wallet.eth.address}
        CHAIN INFO: {JSON.stringify(root_store.wallet.eth.chainInfo, null, 2)}
      </P>
    </Box>
  )
}

export default observer(Home)
