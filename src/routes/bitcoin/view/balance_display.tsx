import { P, Row } from '../../../shortcuts'
import { display_sat, sat2usd } from '../utils/amount_formatters'

export const BalanceDisplay = ({
  satoshis,
  usd_price,
  label = 'Balance',
}: {
  satoshis: string | bigint
  usd_price: number
  label?: string
}) => (
  <Row width={'fit-content'}>
    <P>
      {label} {display_sat(satoshis)} ~
    </P>
    <P level="body-xs">{sat2usd(satoshis, usd_price)}</P>
  </Row>
)
