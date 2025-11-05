import { useEffect, useState } from 'react'
import { commands, SyncStatus } from '../../bindings'
import { notifier } from '../../components/notifier'

export const BitcoinSync = () => {
  const [syncStatus, setSyncStatus] = useState<SyncStatus | null>(null)

  useEffect(() => {
    const interval = setInterval(async () => {
      const r = await commands.chainStatus()
      if (r.status === 'error') {
        notifier.err(r.error)
        return
      }
      setSyncStatus(r.data)
    }, 1000)
    return () => clearInterval(interval)
  }, [])

  return (
    <main className="container">
      <p>Block height: {syncStatus?.height ?? 'Loading...'}</p>
      <p>
        Sync completed:{' '}
        {syncStatus?.sync_completed ? 'Chain is up to date' : 'Syncing...'}
      </p>
    </main>
  )
}
