import { Add } from '@mui/icons-material'
import { IconButton, Input, Option, Select } from '@mui/joy'
import { makeAutoObservable, runInAction } from 'mobx'
import { observer } from 'mobx-react-lite'
import { type BlockChain, commands } from '../bindings'
import { notifier } from '../lib/notifier'
import { P, Row } from '../shortcuts'
import { Loader } from '../stores/loader'

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

export class AccountSelectorVM {
  readonly create = new CreateAccountVM()
  accounts: Account[] = []

  active_account: Account['index'] | null = null
  set_active_account(a: Account['index'] | null) {
    this.active_account = a
  }

  constructor(readonly chain: BlockChain) {
    makeAutoObservable(this)
  }

  init(accounts: Account[], active: Account['index']) {
    this.accounts = accounts
    this.active_account = active
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
  }

  handle_plus_button_click() {
    if (this.create.name_input) {
      this.create_account()
    } else {
      this.create.set_name_input_visible(true)
    }
  }
}

export const AccountSelector = observer(({ vm }: { vm: AccountSelectorVM }) => (
  <Row alignItems={'center'}>
    <P level="body-xs">Account</P>
    <Select
      variant="plain"
      color="primary"
      value={vm.active_account}
      onChange={(_, v) => vm.set_active_account(v)}
      sx={{ width: 'min-content' }}
      size="sm"
      slotProps={{ listbox: { variant: 'soft' } }}
    >
      {vm.accounts.map(each => (
        <Option value={each.index} key={each.index}>
          {each.name}
        </Option>
      ))}

      <Row gap={0.5} px={1}>
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
            minWidth: '24px',
            minHeight: '24px',
          }}
          size="sm"
          onClick={() => vm.handle_plus_button_click()}
        >
          <Add size="sm" />
        </IconButton>
      </Row>
    </Select>
  </Row>
))
