import { Button, Modal, ModalClose, ModalDialog, Stack, Table } from '@mui/joy'
import { makeAutoObservable } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { commands, type Utxo } from '../../bindings'
import { CompactSrt } from '../../components/compact_str'
import { notifier } from '../../lib/notifier'
import { P, Progress, Row } from '../../shortcuts'
import { Loader } from '../../stores/loader'
import { root_store } from '../../stores/root'
import { display_sat, sat2usd } from './utils/amount_formatters'

class UtxoList {
  readonly loader = new Loader()
  constructor() {
    makeAutoObservable(this)
  }

  isOpen = false
  setIsOpen(o: boolean) {
    this.isOpen = o
  }
  utxos: Utxo[] = []

  async fetch() {
    this.loader.start()
    const res = await commands.btcListUtxos()
    if (res.status === 'error') {
      notifier.err(res.error)
      throw new Error(res.error)
    }
    this.utxos = res.data
    this.loader.stop()
  }

  get total_value_sat() {
    return this.utxos.reduce((acc, utxo) => acc + BigInt(utxo.value), 0n)
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
          <P level="h3">Unspent transaction outputs</P>

          {store.loader.loading && <Progress />}

          <Stack sx={{ overflow: 'auto', mt: 1 }}>
            {store.utxos.length === 0 ? (
              <P>No utxos yet.</P>
            ) : (
              <>
                <P>Total {display_sat(store.total_value_sat)}</P>

                <Table variant="plain" stickyHeader>
                  <thead>
                    <tr>
                      <th>
                        <P>Derivation path</P>
                      </th>
                      <th>
                        <P>Label</P>
                      </th>
                      <th>
                        <P>Transaction ID</P>
                      </th>
                      <th>
                        <P>Amount</P>
                      </th>
                      <th>
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
                            <CompactSrt val={utxo.utxo_id.tx_id} />
                          </td>
                          <td>
                            <P sx={{ fontFamily: 'monospace' }}>
                              {display_sat(utxo.value)}
                            </P>
                          </td>
                          <td>
                            <P sx={{ fontFamily: 'monospace' }}>
                              {sat2usd(
                                utxo.value,
                                root_store.wallet.btc.usd_price,
                              )}
                            </P>
                          </td>
                        </tr>
                      )
                    })}
                  </tbody>
                </Table>
              </>
            )}
          </Stack>
        </ModalDialog>
      </Modal>
    </Row>
  )
})
