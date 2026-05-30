import { openUrl } from '@tauri-apps/plugin-opener'
import { B } from '../../../shortcuts'

type OpenArgs = {
  type: 'tx'
  txid: string
}

const get_url = (args: OpenArgs): string => {
  switch (args.type) {
    case 'tx':
      return `https://mempool.space/tx/${args.txid}`
  }
}

export const ExplorerLink = (props: OpenArgs) => (
  <B variant="soft" onClick={() => openUrl(get_url(props))}>
    Open explorer
  </B>
)
