import { Card } from '@mui/joy'
import { P, Row } from '../shortcuts'

export const Address = (props: { addr: string }) => (
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
  </Card>
)
