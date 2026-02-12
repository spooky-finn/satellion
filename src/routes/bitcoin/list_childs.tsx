import { Button, Modal, ModalClose, ModalDialog, Stack, Table } from '@mui/joy'
import { makeAutoObservable } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { commands, type DerivedAddress } from '../../bindings'
import { CompactSrt } from '../../components/compact_str'
import { notifier } from '../../lib/notifier'
import { P, Progress, Row } from '../../shortcuts'
import { Loader } from '../../stores/loader'

class ChildAddressList {
  readonly loader = new Loader()

  constructor() {
    makeAutoObservable(this)
  }

  isOpen = false
  setIsOpen(o: boolean) {
    this.isOpen = o
  }

  addresses: DerivedAddress[] = []

  async fetch() {
    this.loader.start()
    const res = await commands.btcListDerivedAddresess()
    this.loader.stop()
    if (res.status === 'error') {
      notifier.err(res.error)
      return
    }
    this.addresses = res.data
  }
}

export const ListDerivedAddresses = observer(() => {
  const [store] = useState(() => new ChildAddressList())

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
        List childs
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
                        <P fontFamily="monospace">{addr.deriv_path}</P>
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
