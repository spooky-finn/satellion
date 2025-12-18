import { Button, Card, Container, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { Navbar } from '../../components/navbar'
import { route, useNavigate } from '../../routes'
import { P, Row } from '../../shortcuts'
import { store } from './store'

const GenerateMnemonic = observer(() => {
  const [isCopied, setIsCopied] = useState(false)
  const navigate = useNavigate()
  useEffect(() => {
    store.generate()
  }, [])
  if (store.mnemonic.length === 0) {
    return null
  }
  return (
    <Stack gap={1} alignItems={'center'}>
      <Navbar hideLedgers />
      <Container maxWidth="md">
        <P level="h2">Random secret private key</P>
        <Stack>
          <Card size="sm" variant="soft">
            <Row
              gap={3}
              alignItems={'center'}
              onDoubleClick={() => {
                navigator.clipboard.writeText(store.mnemonic.join(' '))
                setIsCopied(true)
              }}
            >
              <P level="body-lg">{store.mnemonic.join(' ')}</P>
            </Row>
          </Card>
          {isCopied && (
            <P level="body-xs" color="success" textAlign={'center'}>
              Copied
            </P>
          )}
        </Stack>

        <Stack gap={1}>
          <Button
            variant="soft"
            color="neutral"
            onClick={() => store.generate()}
          >
            Regenerate
          </Button>
          <Button
            variant="soft"
            color="primary"
            onClick={() => navigate(route.verify_mnemonic)}
          >
            Continue
          </Button>
        </Stack>
      </Container>
    </Stack>
  )
})

export default GenerateMnemonic
