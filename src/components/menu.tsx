import SettingsIcon from '@mui/icons-material/Settings'
import { Chip, Dropdown, Menu, MenuButton, MenuItem } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { useState } from 'react'
import { useNavigate } from 'react-router'
import { route } from '../lib/routes'
import { root_store } from '../view_model/root'
import { TorSettings } from './tor_settings'

export const AppMenu = observer(() => {
  const navigate = useNavigate()
  const [torOpen, setTorOpen] = useState(false)
  const torEnabled = root_store.ui_config?.tor_enabled ?? false

  return (
    <>
      <Dropdown>
        <MenuButton size="sm" color="neutral" variant="plain">
          <SettingsIcon />
          {torEnabled && (
            <Chip
              size="sm"
              color="success"
              variant="solid"
              sx={{ ml: 0.5, fontSize: 9, px: 0.5 }}
            >
              TOR
            </Chip>
          )}
        </MenuButton>
        <Menu>
          <MenuItem onClick={() => setTorOpen(true)}>Tor settings</MenuItem>
          {root_store.wallet.name && (
            <MenuItem
              onClick={async () => {
                await root_store.wallet.forget(root_store.wallet.name!)
                navigate(route.unlock_wallet)
              }}
            >
              Forget wallet
            </MenuItem>
          )}
        </Menu>
      </Dropdown>

      <TorSettings open={torOpen} onClose={() => setTorOpen(false)} />
    </>
  )
})
