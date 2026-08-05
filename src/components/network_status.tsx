import LayersRoundedIcon from '@mui/icons-material/LayersRounded'
import { P, Row } from '../shortcuts'
import { Copyable } from './copyable'

export interface NetworkStatusProps {
  height?: number | string | null
  hash?: string | null
  price?: number | string | null
}

export const NetworkStatus = ({ p }: { p: NetworkStatusProps }) => {
  const { price, hash, height } = p
  return (
    <Row sx={{ fontSize: '.5rem', alignItems: 'center' }} gap={1}>
      <LayersRoundedIcon
        sx={{ color: 'neutral.plainColor', fontSize: '1rem' }}
      />
      {height && <P level="body-xs">height = {height}</P>}
      {hash && (
        <P level="body-xs" display={'flex'} alignItems={'baseline'}>
          hash =
          <Copyable val={hash} level="body-xs" copy />
        </P>
      )}
      {price && <P level="body-xs">price = ${price}</P>}
    </Row>
  )
}
