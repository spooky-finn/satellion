import { Button, Input, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { Navbar } from '../../components/navbar'
import { route, useNavigate } from '../../routes'
import { P, Row } from '../../shortcuts'
import { store } from './store'

const VerifyMnemonic = observer(() => {
  const navigate = useNavigate()
  return (
    <Stack gap={3} alignItems={'center'}>
      <Navbar hideLedgers />
      {store.verificationIndices.map(index => (
        <Row key={index}>
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

      <Button
        variant="soft"
        color="neutral"
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
      {store.verificationSuccessfull ===
      null ? null : store.verificationSuccessfull ? (
        <P>Verification successful</P>
      ) : (
        <P>Verification failed</P>
      )}
    </Stack>
  )
})

export default VerifyMnemonic
