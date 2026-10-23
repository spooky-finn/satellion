<script lang="ts">
  import { onMount } from 'svelte'
  import { isTauri } from '@tauri-apps/api/core'
  import { listen } from '@tauri-apps/api/event'
  import { theme } from './lib/theme.svelte'
  import { root_store } from './view_model/root.svelte'
  import { route } from './lib/routes'
  import { router } from './lib/router.svelte'
  import { handle_err } from './lib/handle_err'
  import UnlockWallet from './routes/UnlockWallet.svelte'
  import CreateWallet from './routes/CreateWallet.svelte'
  import Settings from './routes/Settings.svelte'
  import BitcoinWallet from './routes/bitcoin/view/Main.svelte'
  import EthereumWallet from './routes/ethereum/view/Main.svelte'
  import Notifications from './components/Notifications.svelte'
  import ThemeSwitcher from './components/ThemeSwitcher.svelte'
  import Navbar from './components/Navbar.svelte'

  const desktop = isTauri()
  const authenticated = $derived(
    root_store.unlock.is_unlocked && !!root_store.wallet.name,
  )
  const walletPath = $derived(
    router.path === route.bitcoin || router.path === route.ethereum,
  )

  function lock() {
    root_store.wallet.reset()
    root_store.unlock.reset()
    router.navigate(route.unlock_wallet, true)
  }

  function keyboard(event: KeyboardEvent) {
    if (event.key === 'Escape' && router.path === route.settings) router.back()

    if (
      authenticated &&
      (event.key === 'F5' ||
        ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'r'))
    ) {
      event.preventDefault()

      if (authenticated && router.path === route.bitcoin) {
        root_store.wallet.btc.load_account_info().catch(handle_err)
        root_store.wallet.btc.load_chain_tip().catch(handle_err)
      } else if (authenticated && router.path === route.ethereum)
        root_store.wallet.eth.load_active_account().catch(handle_err)
    }
  }

  onMount(() => {
    const stopTheme = theme.watch()

    if (!desktop) return stopTheme

    root_store.bootstrap().catch(handle_err)

    if (!authenticated && walletPath) router.navigate(route.unlock_wallet, true)

    let disposed = false
    let unlisten: (() => void) | undefined

    listen('session_expired', lock)
      .then(stop => {
        if (disposed) stop()
        else unlisten = stop
      })
      .catch(handle_err)

    return () => {
      stopTheme()
      disposed = true
      unlisten?.()
    }
  })
</script>

<svelte:window onpopstate={() => router.sync()} onkeydown={keyboard} />
<div class="app-shell stack compact">
  {#if desktop && authenticated && walletPath}
    <Navbar />
  {/if}
  {#if !desktop}
    <section class="auth panel stack">
      <h1>Satellion Wallet</h1>
      <p>Open Satellion in the desktop app to access your wallets.</p>
      <p class="muted">
        The frontend preview is running. Wallet operations require the Tauri
        desktop app.
      </p>
      <ThemeSwitcher />
    </section>
  {:else if router.path === route.create_wallet}
    <CreateWallet />
  {:else if authenticated && router.path === route.settings}
    <Settings />
  {:else if authenticated && router.path === route.bitcoin}
    <BitcoinWallet />
  {:else if authenticated && router.path === route.ethereum}
    <EthereumWallet />
  {:else}
    <UnlockWallet />
  {/if}
</div>
<Notifications />
