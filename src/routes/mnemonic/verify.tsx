import { Button, Input, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { route, useNavigate } from '../../routes'
import { P, Row } from '../../shortcuts'
import { store } from './store'

const VerifyMnemonic = observer(() => {
  const navigate = useNavigate()
  return (
    <Stack gap={3}>
      {store.verificationIndices.map(index => (
        <Row>
          <P>{index + 1}</P>
          <Input
            key={index}
            value={store.verificationWords[index]}
            onChange={e => {
              store.setVirificationWords(index, e.target.value)
            }}
          />
        </Row>
      ))}

      <Button
        sx={{ width: 'min-content' }}
        onClick={() => {
          const status = store.verifyMnemonic()
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
