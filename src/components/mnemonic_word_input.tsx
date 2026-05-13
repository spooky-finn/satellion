import { Autocomplete, createFilterOptions } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { P, Row } from '../shortcuts'
import { root_store } from '../view_model/root'

const INPUT_WIDTH = 150

export const MnemonicWordInput = observer(
  (props: {
    id: number
    value: string
    visible: boolean
    onChange: (v: string | null) => void
    onFocus?: React.FocusEventHandler<HTMLInputElement>
    onPaste?: React.ClipboardEventHandler<HTMLInputElement>
  }) => {
    const { id } = props
    return (
      <Row alignItems={'center'}>
        <P level="body-xs" width={10} textAlign={'end'}>
          {id + 1}
        </P>
        <Autocomplete
          sx={{ width: INPUT_WIDTH }}
          size="sm"
          variant="outlined"
          type={props.visible ? 'text' : 'password'}
          onFocus={props.onFocus}
          options={root_store.mnemonic_wordlist}
          onChange={(_, v) => props.onChange(v)}
          autoSelect
          autoHighlight
          filterOptions={createFilterOptions({ matchFrom: 'start' })}
          slotProps={{
            input: {
              onPaste: props.onPaste,
            },
          }}
        />
      </Row>
    )
  },
)
