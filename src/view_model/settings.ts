import { makeAutoObservable, runInAction } from 'mobx'
import { type Config, commands } from '../bindings'
import type { FieldSchema } from '../components/config_form'
import { unwrap_result } from '../lib/handle_err'
import { notifier } from '../lib/notifier'
import type { BiometricVM } from './biometric'
import type { Wallet } from './wallet'

export class SettingsVM {
  private wallet: Wallet
  private biometric: BiometricVM

  config: Config | null = null
  schema: FieldSchema | null = null
  restart_hint = false
  rename_draft = ''

  constructor(wallet: Wallet, biometric: BiometricVM) {
    this.wallet = wallet
    this.biometric = biometric
    makeAutoObservable(this)
  }

  get biometric_supported(): boolean {
    return this.biometric.is_supported
  }

  get biometric_enabled(): boolean {
    return this.wallet.name
      ? this.biometric.is_enabled_for(this.wallet.name)
      : false
  }

  get biometric_busy(): boolean {
    return this.biometric.busy
  }

  set_value(path: string[], value: unknown) {
    if (!this.config) return
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let obj: any = this.config
    for (let i = 0; i < path.length - 1; i++) {
      obj = obj[path[i]]
    }
    obj[path[path.length - 1]] = value
    this.restart_hint = false
  }

  async load_settings() {
    const [config, schemaStr] = await Promise.all([
      commands.getConfig().then(unwrap_result),
      commands.getConfigSchema().then(unwrap_result),
    ])
    runInAction(() => {
      this.config = config
      this.schema = JSON.parse(schemaStr) as FieldSchema
      if (this.wallet.name !== undefined) this.rename_draft = this.wallet.name
    })
    if (this.wallet.name) await this.biometric.refresh(this.wallet.name)
  }

  async toggle_biometric(enable: boolean) {
    if (!this.wallet.name) return
    const ok = enable
      ? await this.biometric.enable_current(this.wallet.name)
      : await this.biometric.disable(this.wallet.name)
    if (ok) {
      notifier.ok(
        enable ? 'Touch ID unlock enabled' : 'Touch ID unlock disabled',
      )
    }
  }

  async request_config() {
    const config = await commands.getConfig().then(unwrap_result)
    runInAction(() => {
      this.config = config
    })
  }

  async save() {
    if (!this.config) return
    const res = await commands.setConfig(this.config)
    if (res.status === 'error') {
      notifier.err(res.error)
      return
    }
    runInAction(() => {
      this.restart_hint = true
    })
  }

  set_rename_draft(v: string) {
    this.rename_draft = v
  }

  async rename(): Promise<string | null> {
    const r = await commands.renameWallet(this.rename_draft.trim())
    if (r.status === 'error') {
      notifier.err(r.error)
      return null
    }

    runInAction(() => {
      this.rename_draft = r.data
    })

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
