import { Button, Card, Container, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { Navbar } from '../../../components/navbar'
import { NavigateUnlock, P, Row } from '../../../shortcuts'
import { store } from '../mnemonic_store'
import type { FlowState } from './flow_state'

export const GenerateMnemonic = observer(({ flow }: { flow: FlowState }) => {
  const [isCopied, setIsCopied] = useState(false)

  useEffect(() => {
    store.generate()
  }, [])

  if (store.mnemonic.length === 0) return null

  return (
    <Stack gap={1} alignItems={'center'}>
      <Navbar hideLedgers />
      <Container maxWidth="md">
        <Stack gap={1} alignItems={'center'}>
          <P level="h2">Random secret private key</P>
          <Stack gap={1}>
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

          <Row flexWrap={'wrap'}>
            <NavigateUnlock />
            <Button
              variant="plain"
              color="neutral"
              onClick={() => store.generate()}
            >
              Regenerate
            </Button>
            <Button
              variant="soft"
              color="primary"
              onClick={() => flow.set_stage('verify_mnemonic')}
            >
              Continue
            </Button>
          </Row>
        </Stack>
      </Container>
    </Stack>
  )
})
