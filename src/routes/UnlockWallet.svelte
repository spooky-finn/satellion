<script lang="ts">
  import { onMount } from 'svelte'
  import { root_store } from '../view_model/root.svelte'
  import { router } from '../lib/router.svelte'
  import { route } from '../lib/routes'
  import Loading from '../components/Loading.svelte'
  import Icon from '../components/Icon.svelte'
  import { UnlockPage } from './wallet/unlock/unlock_page.svelte'
  import WalletPicker from './wallet/unlock/WalletPicker.svelte'
  import PassphraseForm from './wallet/unlock/PassphraseForm.svelte'

  const { unlock, biometric, wallet } = root_store
  const page = new UnlockPage(
    unlock,
    biometric,
    wallet,
    chain => {
      root_store.on_unlock()
      router.navigate(
        chain === 'Bitcoin' ? route.bitcoin : route.ethereum,
        true,
      )
    },
    () => router.navigate(route.create_wallet, true),
  )
  const loading = $derived(page.loading)
  const error = $derived(page.error)

  onMount(() => {
    page.start()

    return () => page.dispose()
  })
</script>

<svelte:window onkeydown={(event) => page.keyboard(event)} />

<section class="login-page">
  <div class="login-content stack">
    <div class="login-identity stack compact">
      <span class="login-key"><Icon name="key" size={64} /></span>
      {#if unlock.target_wallet}
        <h1>{unlock.target_wallet}</h1>
      {:else if !loading}
        <p class="muted">Select a wallet</p>
      {/if}
    </div>
    {#if loading}
      <Loading label="Loading wallets…" />
    {:else if unlock.loader.loading}
      <Loading label="Inspecting blockchain…" />
    {:else if unlock.target_wallet}
      {#key unlock.target_wallet}
        <PassphraseForm {unlock} onsubmit={() => page.passwordUnlock()} />
      {/key}
      {#if biometric.is_enabled_for(unlock.target_wallet)}
        <button
          class="btn btn-sm preset-tonal-surface"
          onclick={() => page.biometricUnlock()}
        >
          <Icon name="fingerprint" />
          Use Touch ID
        </button>
      {/if}
    {/if}
    {#if error}
      <p role="alert" class="error">{error}</p>
      <button
        class="btn btn-sm preset-tonal-surface"
        onclick={() => page.load()}
      >
        Reload wallets
      </button>
    {/if}
  </div>
  <WalletPicker
    wallets={unlock.available_wallets}
    selected={unlock.target_wallet}
    disabled={loading || unlock.loader.loading}
    onselect={(name) => page.selectWallet(name)}
    onadd={() => router.navigate(route.create_wallet)}
  />
</section>

<style>
  .login-page {
    min-height: calc(100dvh - 32px);
    padding-bottom: max(96px, calc(35dvh + 16px));
  }

  .login-content {
    align-items: center;
    width: min(100%, 360px);
    margin: 0 auto;
    padding-top: 64px;
    text-align: center;
  }

  .login-identity {
    align-items: center;
  }

  .login-identity h1 {
    max-width: 100%;
    overflow-wrap: anywhere;
  }

  .login-key {
    flex-shrink: 0;
    color: var(--app-icon);
    height: 64px;
  }

  @media (max-height: 450px) {
    .login-content {
      padding-top: 16px;
    }
  }
</style>
