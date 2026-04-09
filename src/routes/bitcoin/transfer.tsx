import { Stack } from '@mui/joy'
import { useState } from 'react'
import { Navbar } from '../../components/navbar'
import { P } from '../../shortcuts'
import { AddressInput } from '../components'
import { BitcoinTransferVM } from './transfer.vm'

export const BitcoinTransfer = () => {
  const [state] = useState(() => new BitcoinTransferVM())
  return (
    <Stack gap={1}>
      <Navbar />
      <P level="h3" color="primary">
        Send bitcoin
      </P>
      <AddressInput state={state.address} />
    </Stack>
  )
}
