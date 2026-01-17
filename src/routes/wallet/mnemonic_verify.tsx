import { Button, Container, Input, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Navbar } from '../../components/navbar'
import { NavigateUnlock, P, Row } from '../../shortcuts'
import type { State } from './mnemonic_generate'
import { store } from './store'

export const VerifyMnemonic = observer(({ state }: { state: State }) => (
  <Stack gap={1} alignItems={'center'}>
    <Navbar hideLedgers />
    <Container maxWidth="sm">
      <Stack gap={1} alignItems={'center'}>
        <P level="h2">Enter your mnemonic words</P>
        {store.verification_indices.map(index => (
          <Row key={index} alignItems={'center'}>
            <P>{index + 1}</P>
            <Input
              autoComplete="chrome-off"
              autoCorrect="off"
              key={index}
              value={store.verification_words[index]}
              onChange={e => {
                store.set_verification_words(index, e.target.value)
              }}
            />
          </Row>
        ))}

        <Stack gap={1} alignItems={'center'}>
          <Button
            variant="soft"
            color="primary"
            sx={{ width: 'min-content' }}
            onClick={() => {
              const status = store.verify()
              if (status) {
                state.set_stage('set_passphrase')
              }
            }}
          >
            Verify
          </Button>
          <NavigateUnlock />
        </Stack>

        {store.verification_successfull ===
        null ? null : store.verification_successfull ? (
          <P>Verification successful</P>
        ) : (
          <P color="danger">Verification failed</P>
        )}
      </Stack>
    </Container>
  </Stack>
))
