import { Dropdown, Menu, MenuButton, MenuItem } from '@mui/joy'
import { useNavigate } from 'react-router'
import { route } from '../routes'
import { root_store } from '../stores/root'

export const AppMenu = () => {
  const navigate = useNavigate()

  return (
    <Dropdown>
      <MenuButton size="sm" color="neutral" variant="plain">
        Settings
      </MenuButton>
      <Menu>
        <MenuItem
          onClick={() => {
            navigate(route.unlock_wallet)
          }}
        >
          Lock
        </MenuItem>
        <MenuItem
          onClick={async () => {
            await root_store.wallet.forget()
            navigate(route.unlock_wallet)
          }}
        >
          Forget
        </MenuItem>
      </Menu>
    </Dropdown>
  )
}
