import { Button, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { route, useNavigate } from '../routes'
import { P, Row } from '../shortcuts'

const CreateWallet = observer(() => {
  const navigate = useNavigate()

  return (
    <Stack gap={3} alignItems={'center'}>
      <P level="h2" color="primary">
        Add wallet
      </P>
      <Row sx={{ width: 'min-content' }}>
        <Button>Import</Button>
        <Button onClick={() => navigate(route.gen_mnemonic)}>Generate</Button>
      </Row>
    </Stack>
  )
})

export default CreateWallet
