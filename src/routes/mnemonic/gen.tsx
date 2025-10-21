import { Button, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { route, useNavigate } from '../../routes'
import { P } from '../../shortcuts'
import { store } from './store'

const GenerateMnemonic = observer(() => {
  const navigate = useNavigate()
  useEffect(() => {
    store.generateMnemonic()
  }, [])
  if (store.mnemonic.length === 0) {
    return null
  }
  return (
    <Stack gap={2}>
      <P level="h3" color="primary">
        Your private key
      </P>
      <Stack>
        {store.mnemonic.map((word, index) => (
          <P key={index}>
            {index + 1}. {word}
          </P>
        ))}
      </Stack>
      <Button
        sx={{ width: 'min-content' }}
        onClick={() => navigate(route.verify_mnemonic)}
      >
        Continue
      </Button>
    </Stack>
  )
})

export default GenerateMnemonic
