import { Add, Edit } from '@mui/icons-material'
import { IconButton, Input, Option, Select, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import type { MouseEvent } from 'react'
import { P, Row } from '../../shortcuts'
import { Copyable } from '../copyable'
import type { Account, AccountSelectorVM } from './store'

export const AccountSelector = observer(({ vm }: { vm: AccountSelectorVM }) => {
  const renaming = vm.rename.account != null

  return (
    <Row alignItems={'center'} gap={0.5}>
      <P>Account</P>
      {renaming ? <RenameAccountInput vm={vm} /> : <AccountSelect vm={vm} />}
    </Row>
  )
})

const RenameAccountInput = observer(({ vm }: { vm: AccountSelectorVM }) => (
  <Input
    autoFocus
    variant="plain"
    color="primary"
    size="sm"
    value={vm.rename.input}
    onBlur={() => vm.save_rename_account()}
    onChange={e => vm.rename.set_input(e.target.value)}
    onKeyDown={e => {
      if (e.key === 'Enter') {
        e.currentTarget.blur()
      }
      if (e.key === 'Escape') {
        vm.cancel_rename_account()
      }
    }}
    sx={{ minWidth: 130 }}
    disabled={vm.rename.loader.loading}
  />
))

const AccountSelect = observer(({ vm }: { vm: AccountSelectorVM }) => (
  <Select
    variant="plain"
    color="primary"
    value={vm.active_account}
    onChange={(_, v) => {
      vm.set_active_account(v)
      vm.handle_account_switch(v)
    }}
    sx={{ width: 'min-content', gap: 0.5 }}
    size="sm"
    slotProps={{
      listbox: { variant: 'outlined', color: 'neutral', size: 'md' },
    }}
    disabled={vm.account_loader.loading}
  >
    {vm.accounts.map(each => (
      <AccountOption account={each} vm={vm} key={each.index} />
    ))}

    <CreateAccountRow vm={vm} />
  </Select>
))

const AccountOption = ({
  account,
  vm,
}: {
  account: Account
  vm: AccountSelectorVM
}) => (
  <Option
    value={account.index}
    label={account.name}
    sx={{
      '& .account-rename-button': {
        opacity: 0,
        pointerEvents: 'none',
        transition: 'opacity 120ms ease-in-out',
      },
      '&:hover .account-rename-button': {
        opacity: 1,
        pointerEvents: 'auto',
      },
      '&:focus-within .account-rename-button': {
        opacity: 1,
        pointerEvents: 'auto',
      },
    }}
  >
    <Row
      alignItems={'center'}
      justifyContent={'space-between'}
      gap={1}
      sx={{ width: '100%' }}
    >
      <Stack sx={{ minWidth: 0 }}>
        <Row alignItems={'center'}>
          <P level="body-xs">[{account.index}]</P> {account.name}
        </Row>
        <Copyable
          copy
          sx={{
            fontSize: '.8rem',
            fontWeight: 700,
          }}
          val={account.address}
        />
      </Stack>
      <RenameAccountButton account={account} vm={vm} />
    </Row>
  </Option>
)

const RenameAccountButton = ({
  account,
  vm,
}: {
  account: Account
  vm: AccountSelectorVM
}) => {
  const stop_select = (e: MouseEvent) => {
    e.preventDefault()
    e.stopPropagation()
  }

  return (
    <IconButton
      className="account-rename-button"
      variant="plain"
      color="neutral"
      size="sm"
      disabled={vm.rename.loader.loading}
      onMouseDown={stop_select}
      onClick={e => {
        stop_select(e)
        vm.start_rename_account(account)
      }}
    >
      <Edit sx={{ fontSize: 18 }} />
    </IconButton>
  )
}

const CreateAccountRow = observer(({ vm }: { vm: AccountSelectorVM }) => (
  <Row gap={0.5} px={1} pt={0.5}>
    {vm.create.name_input_visible && (
      <Input
        sx={{ minWidth: '50px' }}
        size="sm"
        placeholder="Account name"
        value={vm.create.name_input}
        onChange={e => {
          vm.create.set_name_input(e.target.value)
        }}
      />
    )}
    <IconButton
      loading={vm.create.loader.loading}
      variant="plain"
      color="neutral"
      sx={{
        width: 'min-content',
        ml: 'auto',
      }}
      size="md"
      onClick={() => vm.handle_plus_button_click()}
    >
      <Add size="sm" />
    </IconButton>
  </Row>
))
