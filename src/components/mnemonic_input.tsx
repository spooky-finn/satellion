import { Box, Card } from '@mui/joy'
import { makeAutoObservable } from 'mobx'
import { observer } from 'mobx-react-lite'
import { P } from '../shortcuts'
import { MnemonicWordInput } from './mnemonic_word_input'

export class MnemonicInputSt {
  constructor(private word_count = 12) {
    makeAutoObservable(this)
  }

  get input_identifiers(): number[] {
    return Array.from(
      {
        length: this.word_count,
      },
      (_, i) => i,
    )
  }

  selected_window = 0
  set_selected_window(id: number) {
    this.selected_window = id
  }

  mnemonic: string[] = []
  set_word(id: number, w: string) {
    this.mnemonic[id] = w.trim().toLocaleLowerCase()
  }

  handle_paste(e: React.ClipboardEvent<HTMLInputElement>) {
    const pastedText = e.clipboardData.getData('text')
    const words = pastedText
      .trim()
      .split(/\s+/)
      .filter(w => w.length > 0)

    if (words.length >= 12) {
      e.preventDefault()
      words.forEach((word, idx) => {
        this.set_word(idx, word)
      })
    }
  }

  get is_input_completed() {
    const words = this.mnemonic.filter(w => w.length > 0)
    return [12, 24].includes(words.length)
  }
}

export const MnemonicInput = observer(({ st }: { st: MnemonicInputSt }) => (
  <Card size="sm" variant="soft">
    <P color="neutral" level="body-xs" textAlign={'center'}>
      Enter your mnemonic phrase
    </P>
    <Box
      py={1}
      gap={0.5}
      sx={{ display: 'flex', flexWrap: 'wrap', justifyContent: 'center' }}
    >
      {st.input_identifiers.map(id => (
        <MnemonicWordInput
          id={id}
          key={id}
          value={st.mnemonic[id]}
          onFocus={() => st.set_selected_window(id)}
          onChange={e => st.set_word(id, e.target.value)}
          visible={st.selected_window === id}
          onPaste={e => st.handle_paste(e)}
        />
      ))}
    </Box>
  </Card>
))
