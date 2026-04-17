import { Stack, Table } from '@mui/joy'
import { makeAutoObservable, runInAction } from 'mobx'
import { observer } from 'mobx-react-lite'
import { commands, type DerivedAddressDto } from '../../bindings/btc'
import { CompactSrt } from '../../components/compact_str'
import { unwrap_result } from '../../lib/handle_err'
import { FullScreenModal, P, Progress, Row } from '../../shortcuts'
import { Loader } from '../../stores/loader'
import { root_store } from '../../stores/root'
import { DeriveChildAddress } from './derive_child'

export class ChildAddressListVM {
  readonly loader = new Loader()

  constructor() {
    makeAutoObservable(this)
  }

  is_open = false
  set_open(o: boolean) {
    this.is_open = o
  }

  addresses: DerivedAddressDto[] = []

  async fetch() {
    const addresses = await commands.getExternalAddresess().then(unwrap_result)
    runInAction(() => {
      this.addresses = addresses
    })
  }
}

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
