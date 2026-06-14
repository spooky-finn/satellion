import { Stack } from '@mui/joy'
import { makeAutoObservable } from 'mobx'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import { B, NavigateUnlock, P, Row } from '../shortcuts'
import { GenerateMnemonicFlow } from './wallet/gen/flow'
import { ImportMnemonic } from './wallet/import'

type Flow = 'import' | 'gen'

class State {
  constructor() {
    makeAutoObservable(this)
  }

  flow?: Flow
  set_flow(f: Flow) {
    this.flow = f
  }
}

export const CreateWallet = observer(() => {
  const [state] = useState(() => new State())
  switch (state.flow) {
    case 'gen':
      return <GenerateMnemonicFlow />
    case 'import':
      return <ImportMnemonic />
    default:
      return <SelectFlow state={state} />
  }
})

const SelectFlow = observer(({ state }: { state: State }) => (
  <Stack gap={2} alignItems={'center'}>
    <P level="h2">Add wallet</P>
    <Row sx={{ width: 'min-content' }}>
      <B
        variant="soft"
        color="neutral"
        onClick={() => state.set_flow('import')}
      >
        Import
      </B>
      <B variant="soft" color="neutral" onClick={() => state.set_flow('gen')}>
        Generate
      </B>
    </Row>
    <NavigateUnlock />
  </Stack>
))
