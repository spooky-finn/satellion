import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { route } from '../routes'
import { root_store } from './store/root'

const Home = () => {
  const navigate = useNavigate()

  useEffect(() => {
    if (!root_store.unlock.unlocked) {
      navigate(route.unlock_wallet)
    }
  }, [])

  return <div>Home</div>
}

export default observer(Home)
