import { Button, Container, Input, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Navbar } from '../../components/navbar'
import { route, useNavigate } from '../../routes'
import { NavigateUnlock, P, Row } from '../../shortcuts'
import { store } from './store'

const VerifyMnemonic = observer(() => {
  const navigate = useNavigate()
  return (
    <Stack gap={1} alignItems={'center'}>
      <Navbar hideLedgers />
      <Container maxWidth="sm">
        <Stack gap={1} alignItems={'center'}>
          <P level="h2">Enter your mnemonic words</P>
          {store.verificationIndices.map(index => (
            <Row key={index} alignItems={'center'}>
              <P>{index + 1}</P>
              <Input
                autoComplete="chrome-off"
                autoCorrect="off"
                key={index}
                value={store.verificationWords[index]}
                onChange={e => {
                  store.setVirificationWords(index, e.target.value)
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
                  navigate(route.create_passphrase)
                }
              }}
            >
              Verify
            </Button>
            <NavigateUnlock />
          </Stack>

          {store.verificationSuccessfull ===
          null ? null : store.verificationSuccessfull ? (
            <P>Verification successful</P>
          ) : (
            <P color="danger">Verification failed</P>
          )}
        </Stack>
      </Container>
    </Stack>
  )
})

export default VerifyMnemonic
