import SettingsIcon from '@mui/icons-material/Settings'
import { observer } from 'mobx-react-lite'
import { route } from '../lib/routes'
import { root_store } from '../view_model/root'
import { LinkButton } from '../shortcuts'

export const AppMenu = observer(() => {
  const torEnabled = root_store.ui_config?.tor_enabled ?? false
  return (
    <LinkButton to={route.settings} variant="plain" color={torEnabled ? 'success' : 'neutral'}>
      <SettingsIcon />
    </LinkButton>
  )
})
