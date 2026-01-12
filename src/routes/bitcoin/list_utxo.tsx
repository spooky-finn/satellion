import {
  Button,
  Divider,
  Modal,
  ModalClose,
  ModalDialog,
  Stack,
  Table
} from '@mui/joy'
import { makeAutoObservable } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { commands, type Utxo } from '../../bindings'
import { CuttedString } from '../../components/cutted_str'
import { notifier } from '../../components/notifier'
import { P, Row } from '../../shortcuts'

class UtxoList {
  constructor() {
    makeAutoObservable(this)
  }

  isOpen = false
  setIsOpen(o: boolean) {
    this.isOpen = o
  }
  utxos: Utxo[] = []

  async fetch() {
    const res = await commands.btcListUtxos()
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
        Utxo
      </Button>

      <Modal open={store.isOpen} onClose={() => store.setIsOpen(false)}>
        <ModalDialog
          variant="soft"
          sx={{ pr: 6, minWidth: 300 }}
          size="lg"
          layout="fullscreen"
        >
          <ModalClose />

          <P>Unspent transaction outputs</P>
          <Divider sx={{ my: 1 }} />

          <P>
            Total evaluation {store.total_value_sat} sat ={' '}
            {store.total_value_btc} btc
          </P>

          <Stack sx={{ overflow: 'auto', mt: 1 }}>
            {store.utxos.length === 0 ? (
              <P>No utxos yet.</P>
            ) : (
              <Table variant="soft">
                <thead>
                  <tr>
                    <th align="left">
                      <P>Derivation path</P>
                    </th>
                    <th align="left">
                      <P>Label</P>
                    </th>
                    <th align="left">
                      <P>Transaction ID</P>
                    </th>
                    <th align="right">
                      <P>Value</P>
                    </th>
                  </tr>
                </thead>

                <tbody>
                  {store.utxos.map(utxo => {
                    const key = utxo.utxo_id.tx_id + utxo.utxo_id.vout

                    return (
                      <tr key={key}>
                        <td>
                          <P fontFamily={'monospace'}>{utxo.deriv_path}</P>
                        </td>

                        <td>
                          <P>{utxo.address_label}</P>
                        </td>

                        <td>
                          <CuttedString>{utxo.utxo_id.tx_id}</CuttedString>
                        </td>

                        <td align="right">
                          <P sx={{ fontFamily: 'monospace' }}>
                            {utxo.value} sat
                          </P>
                        </td>
                      </tr>
                    )
                  })}
                </tbody>
              </Table>
            )}
          </Stack>
        </ModalDialog>
      </Modal>
    </Row>
  )
})
