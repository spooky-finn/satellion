import { makeAutoObservable, runInAction } from 'mobx'
import { commands, type UIConfig } from '../bindings'
import { unwrap_result } from '../lib/handle_err'
import { notifier } from '../lib/notifier'
import type { Wallet } from './wallet'

export class SettingsVM {
  private wallet: Wallet

  tor_enabled: boolean = false
  tor_proxy: string = 'socks5://127.0.0.1:9050'
  eth_rpc_url: string = 'https://ethereum-rpc.publicnode.com'
  electrum_server: string = ''
  omit_passphrase: boolean = false
  saved: boolean = false

  eth_anvil: boolean = false
  btc_regtest: boolean = false

  rename_draft: string = ''

  constructor(wallet: Wallet) {
    this.wallet = wallet
    makeAutoObservable(this)
  }

  load(config: UIConfig, walletName?: string) {
    this.tor_enabled = config.tor_enabled
    this.tor_proxy = config.tor_socks5_proxy
    this.eth_rpc_url = config.eth_rpc_url
    this.electrum_server = config.btc_electrum_server ?? ''
    this.omit_passphrase = config.omit_passphrase_on_private_key
    this.eth_anvil = config.eth_anvil
    this.btc_regtest = config.btc_regtest
    this.saved = false
    if (walletName !== undefined) this.rename_draft = walletName
  }

  set_tor_enabled(v: boolean) {
    this.tor_enabled = v
    this.saved = false
  }

  set_tor_proxy(v: string) {
    this.tor_proxy = v
    this.saved = false
  }

  set_eth_rpc_url(v: string) {
    this.eth_rpc_url = v
    this.saved = false
  }

  set_electrum_server(v: string) {
    this.electrum_server = v
    this.saved = false
  }

  set_omit_passphrase(v: boolean) {
    this.omit_passphrase = v
    this.saved = false
  }

  set_rename_draft(v: string) {
    this.rename_draft = v
  }

  async load_settings() {
    const config = await commands.getConfig().then(unwrap_result)
    runInAction(() => this.load(config, this.wallet.name))
  }

  async request_config() {
    const config = await commands.getConfig().then(unwrap_result)
    runInAction(() => this.load(config))
  }

  async save() {
    const res = await commands.setConfig({
      tor_enabled: this.tor_enabled,
      tor_socks5_proxy: this.tor_proxy,
      eth_rpc_url: this.eth_rpc_url,
      btc_electrum_server: this.electrum_server.trim() || null,
      omit_passphrase_on_private_key: this.omit_passphrase,
    })
    if (res.status === 'error') {
      notifier.err(res.error)
      return
    }
    this.saved = true
  }

  async rename(): Promise<string | null> {
    const r = await commands.renameWallet(this.rename_draft.trim())
    if (r.status === 'error') {
      notifier.err(r.error)
      return null
    }
    this.rename_draft = r.data
    return r.data
  }

  async rename_wallet() {
    const newName = await this.rename()
    if (newName)
      runInAction(() => {
        this.wallet.name = newName
      })
    return !!newName
  }
}
