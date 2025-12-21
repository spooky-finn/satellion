import { Button, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Navbar } from '../components/navbar'
import { route, useNavigate } from '../routes'
import { P, Row } from '../shortcuts'

export const CreateWallet = observer(() => {
  const navigate = useNavigate()
  return (
    <Stack gap={2} alignItems={'center'}>
      <Navbar hideLedgers />
      <P level="h2">Add wallet</P>
      <Row sx={{ width: 'min-content' }}>
        <Button
          variant="soft"
          color="neutral"
          onClick={() => navigate(route.import_mnemonic)}
        >
          Import
        </Button>
        <Button
          variant="soft"
          color="neutral"
          onClick={() => navigate(route.gen_mnemonic)}
        >
          Generate
        </Button>
      </Row>
    </Stack>
  )
})
