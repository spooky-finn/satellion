import { Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Navbar } from '../components/navbar'
import { P } from '../shortcuts'

const Bitcoin = () => {
  return (
    <Stack>
      <Navbar />
      <P level="h3" color="primary">
        Bitcoin
      </P>
    </Stack>
  )
}

export default observer(Bitcoin)
