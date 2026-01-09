import { Button, Divider, Modal, ModalClose, ModalDialog } from '@mui/joy'
import { makeAutoObservable } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { commands, type Utxo } from '../../bindings'
import { notifier } from '../../components/notifier'
import { P, Row } from '../../shortcuts'
import { root_store } from '../../stores/root'

class UtxoList {
  constructor() {
    makeAutoObservable(this)
  }

  isOpen = false
  setIsOpen(o: boolean) {
    this.isOpen = o
  }
  utxos: Utxo[] = []

  async fetch(walletName: string) {
    const res = await commands.btcListUtxos(walletName)
    if (res.status === 'error') {
      notifier.err(res.error)
      throw new Error(res.error)
    }
    this.utxos = res.data
  }
}

export const ListUtxo = observer(() => {
  const [store] = useState(() => new UtxoList())

  useEffect(() => {
    if (store.isOpen) {
      store.fetch(root_store.wallet.name!)
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
        Unspend tx outputs
      </Button>
      <Modal open={store.isOpen} onClose={() => store.setIsOpen(false)}>
        <ModalDialog sx={{ pr: 6, minWidth: 300 }}>
          <ModalClose />
          <P> Unspend tx outputs</P>
          <Divider sx={{ my: 1 }} />
          {store.utxos.length === 0 ? (
            <P>No utxos yet.</P>
          ) : (
            store.utxos.map(utxo => (
              <Row
                key={utxo.utxoid.tx_id + utxo.utxoid.vout}
                justifyContent="space-between"
                sx={{ mb: 1 }}
              >
                <P level="body-xs">{utxo.utxoid.tx_id}</P>
                <P level="body-xs">{utxo.utxoid.vout}</P>
                <P sx={{ fontFamily: 'monospace' }}>{utxo.value} sat</P>
              </Row>
            ))
          )}
        </ModalDialog>
      </Modal>
    </Row>
  )
})
