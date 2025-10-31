import { Button, Card, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { Navbar } from '../../components/navbar'
import { route, useNavigate } from '../../routes'
import { P, Row } from '../../shortcuts'
import { store } from './store'

const GenerateMnemonic = observer(() => {
  const navigate = useNavigate()
  useEffect(() => {
    store.generate()
  }, [])
  if (store.mnemonic.length === 0) {
    return null
  }
  return (
    <Stack gap={5} alignItems={'center'}>
      <Navbar hideLedgers />
      <P level="h2">Random secret private key</P>
      <Card size="sm">
        <Row gap={3} alignItems={'center'}>
          <P level="body-lg">{store.mnemonic.join(' ')}</P>
        </Row>
      </Card>

      <Row>
        <Button
          variant="soft"
          color="primary"
          onClick={() => navigate(route.verify_mnemonic)}
        >
          Ready to verify
        </Button>
        <Button variant="soft" color="neutral" onClick={() => store.generate()}>
          Regen
        </Button>
      </Row>
    </Stack>
  )
})

export default GenerateMnemonic
