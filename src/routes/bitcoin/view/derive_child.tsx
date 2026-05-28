import { Divider, Input, Modal, ModalClose, ModalDialog } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import { CompactSrt } from '../../../components/compact_str'
import { NumberInput } from '../../../components/number_input'
import { B, P, Row } from '../../../shortcuts'
import { DeriveChildVM } from '../view_model/derive_child.vm'

export const DeriveChildAddress = observer((props: { refetch: () => void }) => {
  const [state] = useState(() => new DeriveChildVM())
  return (
    <Row alignItems={'center'}>
      <B
        onClick={() => {
          state.set_is_open(true)
          state.next_unused_key_index()
        }}
      >
        Derive
      </B>
      <Modal open={state.is_open} onClose={() => state.set_is_open(false)}>
        <ModalDialog sx={{ pr: 6 }}>
          <ModalClose />
          <P level="h3">Derive child address</P>
          <Row alignItems={'center'}>
            <P>Index</P>
            <NumberInput
              size="sm"
              sx={{ maxWidth: 70 }}
              value={state.index ?? undefined}
              onChange={v => state.set_index(v ?? null)}
            />
          </Row>
          <Input
            sx={{ width: '200px' }}
            size="sm"
            placeholder="label"
            value={state.label}
            onChange={e => state.setLabel(e.target.value)}
          />
          <B
            loading={state.loader.loading}
            disabled={!state.label || !state.index}
            onClick={() => state.derive().then(props.refetch)}
          >
            Derive
          </B>
          <Divider />
          {state.address && <CompactSrt copy val={state.address} />}
        </ModalDialog>
      </Modal>
    </Row>
  )
})
