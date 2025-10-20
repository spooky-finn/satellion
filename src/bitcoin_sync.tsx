import { invoke } from "@tauri-apps/api/core"
import { useEffect, useState } from "react"
import type { SyncStatus } from "./types"

export const BitcoinSync = () => {
  const [syncStatus, setSyncStatus] = useState<SyncStatus | null>(null);

  useEffect(() => {
    // request block height every 1 second
    const interval = setInterval(async () => {
      const status = await invoke<SyncStatus>("chain_status");
      setSyncStatus(status);
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  return (
    <main className="container">
      <p>Block height: {syncStatus?.height ?? "Loading..."}</p>
      <p>Sync completed: {syncStatus?.sync_completed ? "Chain is up to date" : "Syncing..."}</p>
    </main>
  );
}

