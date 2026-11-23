import type { IWalletKit } from '@reown/walletkit'
import type { IKeyValueStorage } from '@walletconnect/keyvaluestorage'

export const disconnected = { code: 6000, message: 'Wallet disconnected' }

/** Keep relay keys and sessions in memory for the current wallet context. */
class MemoryStorage implements IKeyValueStorage {
  private readonly data = new Map<string, unknown>()

  async getKeys() {
    return [...this.data.keys()]
  }

  async getEntries<T>() {
    return [...this.data.entries()] as [string, T][]
  }

  async getItem<T>(key: string) {
    return this.data.get(key) as T | undefined
  }

  async setItem<T>(key: string, value: T) {
    this.data.set(key, value)
  }

  async removeItem(key: string) {
    this.data.delete(key)
  }
}

export async function open_relay(project_id: string, signal: AbortSignal) {
  const [{ Core }, { WalletKit }] = await Promise.all([
    import('@walletconnect/core'),
    import('@reown/walletkit'),
  ])
  if (signal.aborted) return

  const core = new Core({
    projectId: project_id,
    storage: new MemoryStorage(),
    customStoragePrefix: crypto.randomUUID(),
    telemetryEnabled: false,
    logger: 'silent',
  })

  try {
    return await WalletKit.init({
      core,
      metadata: {
        name: 'Satellion',
        description: 'Satellion Ethereum wallet',
        url: 'https://github.com/spooky-finn/satellion',
        icons: [],
      },
    })
  } catch {
    await core.relayer.transportClose().catch(() => {})
    throw new Error('Could not connect to the WalletConnect relay.')
  }
}

export async function close_relay(client: IWalletKit, notify_dapps: boolean) {
  try {
    if (notify_dapps) {
      const sessions = Object.keys(client.getActiveSessions())
      const disconnecting = Promise.allSettled(
        sessions.map(topic =>
          client.disconnectSession({ topic, reason: disconnected }),
        ),
      )
      let timer: ReturnType<typeof setTimeout> | undefined
      const deadline = new Promise<void>(resolve => {
        timer = setTimeout(resolve, 1500)
      })
      await Promise.race([disconnecting, deadline])
      clearTimeout(timer)
    }
  } finally {
    await client.core.relayer.transportClose().catch(() => {})
  }
}
