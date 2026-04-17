import { Chip, Modal, ModalClose, ModalDialog, Stack, Table } from '@mui/joy'
import { makeAutoObservable } from 'mobx'
import { observer } from 'mobx-react-lite'
import { type UtxoDto } from '../../bindings/btc'
import { CompactSrt } from '../../components/compact_str'
import { P, Progress, Row } from '../../shortcuts'
import { Loader } from '../../stores/loader'
import { root_store } from '../../stores/root'
import { display_sat, sat2usd } from './utils/amount_formatters'

export class UtxoListVM {
  readonly loader = new Loader()
  constructor() {
    makeAutoObservable(this)
  }

  is_open = false
  selection_mode: boolean = false

  open(selection_mode?: boolean) {
    this.is_open = true
    this.selection_mode = selection_mode ?? false
  }

  close() {
    this.is_open = false
  }

  utxo: UtxoDto[] = []
  set_utxo(utxo: UtxoDto[]) {
    this.utxo = utxo
  }

  get total_value_sat() {
    return this.utxo.reduce((acc, utxo) => acc + BigInt(utxo.value), 0n)
  }

  selected_utxo: number[] = []
  select_utxo(index: number) {
    this.selected_utxo.push(index)
  }

  reset() {
    this.selected_utxo = []
  }
}

export const ListUtxo = observer(({ store }: { store: UtxoListVM }) => {
  return (
    <Row alignItems="center">
      <Modal open={store.is_open} onClose={() => store.close()}>
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
            {store.utxo.length === 0 ? (
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
                    {store.utxo.map((utxo, index) => {
                      const key = utxo.utxo_id.tx_id + utxo.utxo_id.vout
                      return (
                        <tr
                          key={key}
                          onClick={() =>
                            store.selection_mode && store.select_utxo(index)
                          }
                        >
                          <td>
                            {store.selected_utxo.includes(index) && (
                              <Chip color="primary" size="sm" />
                            )}
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
