import { type BlockChain, commands } from '../../bindings'
import { unwrap_result } from '../../lib/handle_err'
import { Loader } from '../../view_model/loader.svelte'

export interface Account {
  index: number
  name: string
  address?: string
}

class CreateAccountStore {
  readonly loader = new Loader()
  name_input_visible = $state<boolean>(false)
  name_input = $state<string>('')

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
  account = $state<Account['index'] | null>(null)
  input = $state<string>('')

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

/** Coordinates account selection, creation, and inline renaming for one chain. */
export class AccountSelectorVM {
  readonly create = new CreateAccountStore()
  readonly rename = new RenameAccountStore()
  readonly account_loader = new Loader()
  accounts = $state<Account[]>([])
  active_account = $state<Account['index'] | null>(null)

  constructor(
    readonly chain: BlockChain,
    readonly switch_handler: (account: Account['index']) => Promise<void>,
  ) {}

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
    const name = this.create.name_input.trim()

    if (!name || this.create.loader.loading || this.account_loader.loading)
      return

    this.create.loader.start()

    try {
      const index = await commands
        .addAccount(this.chain, name)
        .then(unwrap_result)

      this.accounts.push({ index, name })
      await this.handle_account_switch(index)
      this.create.reset()
    } finally {
      this.create.loader.stop()
    }
  }

  async handle_account_switch(account: Account['index'] | null) {
    if (
      account === null ||
      this.account_loader.loading ||
      account === this.active_account
    )
      return

    this.account_loader.start()

    try {
      await commands.switchAccount(this.chain, account).then(unwrap_result)
      this.active_account = account
      await this.switch_handler(account)
    } finally {
      this.account_loader.stop()
    }
  }

  start_rename_account(account: Account) {
    this.rename.start(account)
  }

  cancel_rename_account() {
    this.rename.reset()
  }

  async save_rename_account() {
    const account = this.rename.account

    if (account == null || this.rename.loader.loading) return

    const name = this.rename.input.trim()
    const current = this.accounts.find(each => each.index === account)

    if (!name || current?.name === name) {
      this.rename.reset()

      return
    }

    this.rename.loader.start()

    try {
      await commands
        .renameAccount(this.chain, account, name)
        .then(unwrap_result)
      this.accounts = this.accounts.map(item =>
        item.index === account ? { ...item, name } : item,
      )
      this.rename.reset()
    } finally {
      this.rename.loader.stop()
    }
  }
}
