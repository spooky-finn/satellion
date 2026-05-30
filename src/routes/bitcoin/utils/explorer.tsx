import { makeAutoObservable } from 'mobx'
import { B } from '../../../shortcuts'

type OpenArgs = {
  type: 'tx'
  txid: string
}

class Explorer {
  constructor() {
    makeAutoObservable(this)
  }

  get_url(args: OpenArgs): string {
    switch (args.type) {
      case 'tx': {
        return `https://mempool.space/tx/${args.type}`
      }
    }
  }
}

const explorer_vm = new Explorer()

export const ExplorerLink = (props: OpenArgs) => (
  <B variant="soft" onClick={() => explorer_vm.get_url(props)}>
    Open explorer
  </B>
)
