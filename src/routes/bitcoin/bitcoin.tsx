import { Button, Card, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { Navbar } from '../../components/navbar'
import { NumberInput } from '../../components/number_input'
import { route } from '../../routes'
import { P, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'

const explorer_url = 'https://mempool.space/address/'

const Bitcoin = () => {
  const navigate = useNavigate()
  const addr = root_store.wallet.btc.address

  useEffect(() => {
    if (!addr) navigate(route.unlock_wallet)
  }, [addr, navigate])

  return (
    <Stack gap={1}>
      <Navbar />
      <P level="h3" color="primary">
        Bitcoin
      </P>
      {addr && (
        <Card size="sm" variant="soft">
          <Row gap={1}>
            <P>Main Address</P>
            <P fontWeight="bold"> {addr}</P>
          </Row>
          <P level="body-xs">
            Do not share this address with untrusted parties who may send
            tainted or illicit coins.
            <br />
            Receiving funds from suspicious sources can link your wallet to
            illegal activity.
            <br />
            For secure acceptance of funds, consider generating dedicated child
            address per transaction.
          </P>
          <DeriveChildAddress />
        </Card>
      )}
    </Stack>
  )
}

const DeriveChildAddress = observer(() => {
  const { childDeriver } = root_store.wallet.btc
  return (
    <Row alignItems={'center'}>
      <NumberInput
        size="sm"
        sx={{ maxWidth: 70 }}
        value={childDeriver.index}
        onChange={v => childDeriver.setIndex(v)}
      />
      <Button
        size="sm"
        variant="soft"
        sx={{ width: 'fit-content' }}
        onClick={() => childDeriver.derive(root_store.wallet.name!)}
      >
        Derive
      </Button>
      <P>{childDeriver.address}</P>
    </Row>
  )
})

export default observer(Bitcoin)
