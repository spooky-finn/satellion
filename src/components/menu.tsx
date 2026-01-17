import SettingsIcon from '@mui/icons-material/Settings'
import { Dropdown, Menu, MenuButton, MenuItem } from '@mui/joy'
import { useNavigate } from 'react-router'
import { route } from '../routes'
import { root_store } from '../stores/root'

export const AppMenu = () => {
  const navigate = useNavigate()
  return (
    <Dropdown>
      <MenuButton size="sm" color="neutral" variant="plain">
        <SettingsIcon />
      </MenuButton>
      <Menu>
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
  )
}
