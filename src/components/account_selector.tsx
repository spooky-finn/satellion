import { Button, Option, Select } from '@mui/joy'
import { makeAutoObservable, runInAction } from 'mobx'
import { commands, type BlockChain } from '../bindings'
import { notifier } from '../lib/notifier'

export interface Account {
  index: number
  name: string
}

export class AccountSelectorVM {
  accounts: Account[] = []

  active_account: Account['index'] | null = null
  set_active_account(a: Account['index'] | null) {
    this.active_account = a
  }

  constructor() {
    makeAutoObservable(this)
  }

  init(accounts: Account[], active: Account['index']) {
    this.accounts = accounts
    this.active_account = active
  }

  async create(chain: BlockChain, account_name: string) {
    const res = await commands.addAccount(chain, account_name)
    if (res.status !== 'ok') {
      notifier.err(res.error)
      throw res.error
    }

    runInAction(() => {
      this.accounts.push({ index: res.data, name: account_name })
      this.set_active_account(res.data)
    })
  }
}

export const AccountSelector = (props: { vm: AccountSelectorVM }) => (
  <Select
    value={props.vm.active_account}
    onChange={(_, v) => props.vm.set_active_account(v)}
    sx={{ width: 'min-content' }}
    size="sm"
  >
    {props.vm.accounts.map(each => (
      <Option value={each.index} key={each.index}>
        {each.name}
      </Option>
    ))}

    <Button variant="soft" color="success">
      New
    </Button>
  </Select>
)
