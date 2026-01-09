import {
  Button,
  Divider,
  Modal,
  ModalClose,
  ModalDialog,
  Stack
} from '@mui/joy'
import { makeAutoObservable } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { commands, type Utxo } from '../../bindings'
import { CuttedString } from '../../components/cutted_str'
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

  get total_value_sat() {
    return this.utxos.reduce((acc, utxo) => acc + Number(utxo.value), 0)
  }
  get total_value_btc() {
    return this.total_value_sat / 10 ** 8
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
        Utxo
      </Button>
      <Modal open={store.isOpen} onClose={() => store.setIsOpen(false)}>
        <ModalDialog sx={{ pr: 6, minWidth: 300 }}>
          <ModalClose />
          <P>Unspent transaction outputs</P>
          <Divider sx={{ my: 1 }} />
          <P>
            Total evaluation {store.total_value_sat} sat ={' '}
            {store.total_value_btc} btc
          </P>

          <Stack sx={{ overflow: 'auto' }}>
            {store.utxos.length === 0 ? (
              <P>No utxos yet.</P>
            ) : (
              store.utxos.map(utxo => (
                <Row
                  key={utxo.utxoid.tx_id + utxo.utxoid.vout}
                  justifyContent="space-between"
                  sx={{ mb: 1 }}
                >
                  <CuttedString level="body-xs">
                    {utxo.utxoid.tx_id}
                  </CuttedString>
                  {/* <P level="body-xs">{utxo.utxoid.vout}</P> */}
                  <P level="body-xs" sx={{ fontFamily: 'monospace' }}>
                    {utxo.value} sat
                  </P>
                </Row>
              ))
            )}
          </Stack>
        </ModalDialog>
      </Modal>
    </Row>
  )
})
