import { Button, Card, Container, Stack } from '@mui/joy'
import { makeAutoObservable } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { Navbar } from '../../components/navbar'
import { P, Row } from '../../shortcuts'
import { VerifyMnemonic } from './mnemonic_verify'
import { CreatePassphrase } from './passphrase_create'
import { store } from './store'

type Stage = 'select_mnemonic' | 'verify_mnemonic' | 'set_passphrase'

export class State {
  constructor() {
    makeAutoObservable(this)
  }

  stage: Stage = 'select_mnemonic'
  set_stage(s: Stage) {
    this.stage = s
  }
}

export const GenerateMnemonicFlow = observer(() => {
  const [state] = useState(() => new State())

  switch (state.stage) {
    case 'select_mnemonic':
      return <GenerateMnemonic state={state} />
    case 'verify_mnemonic':
      return <VerifyMnemonic state={state} />
    case 'set_passphrase':
      return <CreatePassphrase />
  }
})

const GenerateMnemonic = observer(({ state }: { state: State }) => {
  const [isCopied, setIsCopied] = useState(false)

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
              onClick={() => state.set_stage('verify_mnemonic')}
            >
              Continue
            </Button>
          </Stack>
        </Stack>
      </Container>
    </Stack>
  )
})
