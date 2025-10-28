import { Button, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { Navbar } from '../components/navbar'
import { route } from '../routes'
import { root_store } from '../stores/root'

const Home = () => {
  const navigate = useNavigate()

  useEffect(() => {
    if (!root_store.wallet.initialized) {
      navigate(route.unlock_wallet)
    }
  }, [])

  return (
    <Stack spacing={2}>
      <Navbar />
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
