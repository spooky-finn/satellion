import { Button, Divider, Modal, ModalClose, ModalDialog } from '@mui/joy'
import { makeAutoObservable } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { commands, type DerivedAddress } from '../../bindings'
import { notifier } from '../../components/notifier'
import { P, Row } from '../../shortcuts'

class ChildAddressList {
  constructor() {
    makeAutoObservable(this)
  }

  isOpen = false
  setIsOpen(o: boolean) {
    this.isOpen = o
  }
  addresses: DerivedAddress[] = []

  async fetch() {
    const res = await commands.btcListDerivedAddresess()
    if (res.status === 'error') {
      notifier.err(res.error)
      throw new Error(res.error)
    }
    this.addresses = res.data
  }
}

export const ListDerivedAddresses = observer(() => {
  const [store] = useState(() => new ChildAddressList())

  useEffect(() => {
    if (store.isOpen) {
      store.fetch().catch(() => {})
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
        List child addresses
      </Button>
      <Modal open={store.isOpen} onClose={() => store.setIsOpen(false)}>
        <ModalDialog sx={{ pr: 6, minWidth: 300 }}>
          <ModalClose />
          <P>Derived child addresses</P>
          <Divider sx={{ my: 1 }} />
          {store.addresses.length === 0 ? (
            <P>No addresses derived yet.</P>
          ) : (
            store.addresses.map(addr => (
              <Row
                key={addr.address}
                justifyContent="space-between"
                sx={{ mb: 1 }}
              >
                <P>{addr.label}</P>
                <P sx={{ fontFamily: 'monospace' }}>{addr.address}</P>
              </Row>
            ))
          )}
        </ModalDialog>
      </Modal>
    </Row>
  )
})
