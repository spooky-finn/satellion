import { openUrl } from '@tauri-apps/plugin-opener'
import { B } from '../../../shortcuts'
import { explorer_endpoint } from '../constants'

export const OpenExplorerButton = (props: { path: string }) => (
  <B
    variant="soft"
    onClick={() => openUrl(`${explorer_endpoint}/${props.path}`)}
  >
    Open explorer
  </B>
)
