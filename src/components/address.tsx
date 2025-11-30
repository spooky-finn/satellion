import { Card } from '@mui/joy'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { route } from '../routes'
import { OpenExplorerButton } from '../routes/ethereum/utils/shared'
import { P, Row } from '../shortcuts'

export const Address = (props: { addr: string }) => {
  const navigate = useNavigate()

  useEffect(() => {
    if (!props.addr) {
      navigate(route.unlock_wallet)
    }
  }, [props.addr, navigate])

  if (!props.addr) {
    return null
  }

  return (
    <Card size="sm">
      <Row gap={1}>
        <P>Main Address</P>
        <P fontWeight="bold"> {props.addr.toLowerCase()}</P>
      </Row>
      <P level="body-xs">
        Do not share this address with untrusted parties who may send tainted or
        illicit coins.
        <br />
        Receiving funds from suspicious sources can link your wallet to illegal
        activity.
        <br />
        For secure acceptance of funds, consider generating dedicated child
        address per transaction.
      </P>
      <OpenExplorerButton path={`address/${props.addr}`} />
    </Card>
  )
}
