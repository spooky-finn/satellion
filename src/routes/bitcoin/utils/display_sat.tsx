import { P, Row } from '../../../shortcuts'
import { display_sat, sat2usd } from './amount_formatters'

export const DisplaySat = ({
  satoshis,
  usd_price,
  label = 'Balance',
  fraction_digits,
}: {
  satoshis: string | bigint | number
  usd_price: number
  label?: string
  fraction_digits?: number
}) => (
  <Row width={'fit-content'} alignItems={'center'}>
    <P>
      {label} {display_sat(satoshis)} ~
    </P>
    <P>{sat2usd(satoshis, usd_price, fraction_digits)}</P>
  </Row>
)
