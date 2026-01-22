import { Button } from '@mui/joy'
import { openUrl } from '@tauri-apps/plugin-opener'
import { explorer_endpoint } from '../constants'

export const OpenExplorerButton = (props: { path: string }) => (
	<Button
		variant="soft"
		size="sm"
		sx={{ width: 'fit-content' }}
		onClick={() => openUrl(`${explorer_endpoint}/${props.path}`)}
	>
		Open explorer
	</Button>
)
