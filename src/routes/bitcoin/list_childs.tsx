import { Button, Modal, ModalClose, ModalDialog, Stack, Table } from '@mui/joy'
import { makeAutoObservable, runInAction } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { commands, type DerivedAddressDto } from '../../bindings/btc'
import { CompactSrt } from '../../components/compact_str'
import { unwrap_result } from '../../lib/handle_err'
import { P, Progress, Row } from '../../shortcuts'
import { Loader } from '../../stores/loader'
import { DeriveChildAddress } from './derive_child'

class ChildAddressListVM {
  readonly loader = new Loader()

  constructor() {
    makeAutoObservable(this)
  }

  isOpen = false
  setIsOpen(o: boolean) {
    this.isOpen = o
  }

  addresses: DerivedAddressDto[] = []

  async fetch() {
    // this.loader.start()
    const addresses = await commands.getExternalAddresess().then(unwrap_result)
    // .finally(() => this.loader.stop())
    runInAction(() => {
      this.addresses = addresses
    })
  }
}

export const ChildAddresses = observer(() => {
  const [store] = useState(() => new ChildAddressListVM())

  useEffect(() => {
    if (store.isOpen) {
      store.fetch()
    }
  }, [store.isOpen])

  return (
    <Row alignItems="center">
      <Button
        size="sm"
        variant="soft"
        sx={{ width: 'fit-content' }}
        onClick={() => store.setIsOpen(true)}
      >
        Child addresses
      </Button>
      <Modal open={store.isOpen} onClose={() => store.setIsOpen(false)}>
        <ModalDialog
          variant="soft"
          sx={{ pr: 6, minWidth: 300 }}
          size="lg"
          layout="fullscreen"
        >
          <ModalClose />
          <P level="h3">Derived child addresses</P>
          {store.loader.loading && <Progress />}
          <DeriveChildAddress refetch={() => store.fetch()} />
          <Stack sx={{ overflow: 'auto', mt: 1 }}>
            {store.addresses.length === 0 ? (
              <P>No addresses derived yet.</P>
            ) : (
              <Table variant="plain" stickyHeader>
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
                        <CompactSrt val={addr.address} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </Table>
            )}
          </Stack>
        </ModalDialog>
      </Modal>
    </Row>
  )
})
