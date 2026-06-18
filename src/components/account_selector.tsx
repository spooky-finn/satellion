import { Add, Edit } from '@mui/icons-material'
import { IconButton, Input, Option, Select } from '@mui/joy'
import { makeAutoObservable, runInAction } from 'mobx'
import { observer } from 'mobx-react-lite'
import { type BlockChain, commands } from '../bindings'
import { unwrap_result } from '../lib/handle_err'
import { notifier } from '../lib/notifier'
import { P, Row } from '../shortcuts'
import { Loader } from '../view_model/loader'

export interface Account {
  index: number
  name: string
}

class CreateAccountVM {
  readonly loader = new Loader()

  name_unput_visible: boolean = false
  set_name_input_visible(s: boolean) {
    this.name_unput_visible = s
  }

  constructor() {
    makeAutoObservable(this)
  }

  name_input: string = ''
  set_name_input(v: string) {
    this.name_input = v
  }

  reset() {
    this.name_unput_visible = false
    this.name_input = ''
    this.loader.reset()
  }
}

class RenameAccountVM {
  readonly loader = new Loader()

  account: Account['index'] | null = null
  input: string = ''

  constructor() {
    makeAutoObservable(this)
  }

  start(account: Account) {
    this.account = account.index
    this.input = account.name
  }

  set_input(v: string) {
    this.input = v
  }

  reset() {
    this.account = null
    this.input = ''
    this.loader.reset()
  }
}

export class AccountSelectorVM {
  readonly create = new CreateAccountVM()
  readonly rename = new RenameAccountVM()
  readonly account_loader = new Loader()
  accounts: Account[] = []

  active_account: Account['index'] | null = null
  set_active_account(a: Account['index'] | null) {
    this.active_account = a
  }

  constructor(
    readonly chain: BlockChain,
    readonly switch_handler: (account: Account['index']) => Promise<void>,
  ) {
    makeAutoObservable(this)
  }

  init(accounts: Account[], active: Account['index']) {
    this.accounts = accounts
    this.active_account = active
    this.rename.reset()
  }

  get active_account_item() {
    return (
      this.accounts.find(each => each.index === this.active_account) ?? null
    )
  }

  async create_account() {
    this.create.loader.start()
    const account_name = this.create.name_input.trim()

    const res = await commands.addAccount(this.chain, account_name)
    if (res.status !== 'ok') {
      notifier.err(res.error)
      throw res.error
    }

    runInAction(() => {
      this.accounts.push({ index: res.data, name: account_name })
      this.set_active_account(res.data)
      this.create.reset()
    })
    await this.switch_handler(res.data)
  }

  handle_plus_button_click() {
    if (this.create.name_input) {
      this.create_account()
    } else {
      this.create.set_name_input_visible(true)
    }
  }

  async handle_account_switch(account: Account['index'] | null) {
    if (account == null) return

    this.account_loader.start()
    await commands
      .switchAccount(this.chain, account)
      .then(unwrap_result)
      .finally(() => this.account_loader.stop())

    await this.switch_handler(account)
  }

  start_rename_active_account() {
    const active = this.active_account_item
    if (!active) return

    this.rename.start(active)
  }

  cancel_rename_account() {
    this.rename.reset()
  }

  async save_rename_account() {
    const account = this.rename.account
    if (account == null) return

    const name = this.rename.input.trim()
    const current = this.accounts.find(each => each.index === account)
    if (!name || current?.name === name) {
      this.rename.reset()
      return
    }

    this.rename.loader.start()
    const res = await commands.renameAccount(this.chain, account, name)
    if (res.status !== 'ok') {
      notifier.err(res.error)
      this.rename.loader.stop()
      return
    }

    runInAction(() => {
      this.accounts = this.accounts.map(each =>
        each.index === account ? { ...each, name } : each,
      )
      this.rename.reset()
    })
  }
}

export const AccountSelector = observer(({ vm }: { vm: AccountSelectorVM }) => {
  const renaming = vm.rename.account != null

  return (
    <Row alignItems={'center'} gap={0.5}>
      <P>Account</P>
      {renaming ? (
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
      ) : (
        <>
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
              <Option value={each.index} key={each.index}>
                <P level="body-xs">[{each.index}]</P> {each.name}
              </Option>
            ))}

            <Row gap={0.5} px={1} pt={0.5}>
              {vm.create.name_unput_visible && (
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
                }}
                size="md"
                onClick={() => vm.handle_plus_button_click()}
              >
                <Add size="sm" />
              </IconButton>
            </Row>
          </Select>
          <IconButton
            variant="plain"
            color="neutral"
            size="sm"
            disabled={!vm.active_account_item || vm.account_loader.loading}
            onClick={() => vm.start_rename_active_account()}
          >
            <Edit sx={{ fontSize: 18 }} />
          </IconButton>
        </>
      )}
    </Row>
  )
})
