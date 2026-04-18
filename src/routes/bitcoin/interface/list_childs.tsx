import { Stack, Table } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { CompactSrt } from '../../../components/compact_str'
import { FullScreenModal, P, Progress, Row } from '../../../shortcuts'
import { root_store } from '../../../view_model/root'
import { DeriveChildAddress } from './derive_child'

export const ChildAddressesModal = observer(() => {
  const store = root_store.wallet.btc.child_list
  return (
    <FullScreenModal open={store.is_open} onClose={() => store.set_open(false)}>
      <ChildAddresses />
    </FullScreenModal>
  )
})

const ChildAddresses = observer(() => {
  const store = root_store.wallet.btc.child_list
  return (
    <>
      <Row>
        <P level="h3">Child addresses</P>
        <DeriveChildAddress refetch={() => store.fetch()} />
      </Row>
      {store.loader.loading && <Progress />}
      <Stack sx={{ overflow: 'auto' }}>
        {store.addresses.length === 0 ? (
          <P>No addresses derived yet.</P>
        ) : (
          <Table variant="plain" stickyHeader size="sm">
            <thead>
              <tr>
                <th align="left">
                  <P>Label</P>
                </th>
                <th align="left">
                  <P>Derivation path</P>
                </th>
                <th align="left">
                  <P>Address</P>
                </th>
              </tr>
            </thead>
            <tbody>
              {store.addresses.map(addr => (
                <tr key={addr.address}>
                  <td>
                    <P>{addr.label}</P>
                  </td>
                  <td>
                    <P fontFamily="monospace">{addr.path}</P>
                  </td>
                  <td>
                    <CompactSrt val={addr.address} copy />
                  </td>
                </tr>
              ))}
            </tbody>
          </Table>
        )}
      </Stack>
    </>
  )
})
