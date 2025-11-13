import { Button, Card, Link } from '@mui/joy'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'
import { route } from '../routes'
import { P, Row } from '../shortcuts'

export const Address = (props: { addr: string; explorer_url?: string }) => {
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
      {props.explorer_url && (
        <Link href={props.explorer_url} target="_blank">
          <Button variant="plain" size="sm">
            View on Explorer
          </Button>
        </Link>
      )}
    </Card>
  )
}
