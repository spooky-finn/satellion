import { makeAutoObservable, runInAction } from 'mobx'
import { type BlockChain, commands } from '../../bindings'
import { unwrap_result } from '../../lib/handle_err'
import { notifier } from '../../lib/notifier'
import { Loader } from '../../view_model/loader'

export interface Account {
  index: number
  name: string
  address?: string
}

class CreateAccountStore {
  readonly loader = new Loader()

  name_input_visible: boolean = false
  name_input: string = ''

  constructor() {
    makeAutoObservable(this)
  }

  set_name_input_visible(s: boolean) {
    this.name_input_visible = s
  }

  set_name_input(v: string) {
    this.name_input = v
  }

  reset() {
    this.name_input_visible = false
    this.name_input = ''
    this.loader.reset()
  }
}

class RenameAccountStore {
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
  readonly create = new CreateAccountStore()
  readonly rename = new RenameAccountStore()
  readonly account_loader = new Loader()
  accounts: Account[] = []

  active_account: Account['index'] | null = null

  constructor(
    readonly chain: BlockChain,
    readonly switch_handler: (account: Account['index']) => Promise<void>,
  ) {
    makeAutoObservable(this)
  }

  set_active_account(a: Account['index'] | null) {
    this.active_account = a
  }

  set_account_address(account: Account['index'], address: string) {
    this.accounts = this.accounts.map(each =>
      each.index === account ? { ...each, address } : each,
    )
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

  start_rename_account(account: Account) {
    this.rename.start(account)
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
