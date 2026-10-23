<script lang="ts">
  import { onMount } from 'svelte'
  import { root_store } from '../view_model/root.svelte'
  import { router } from '../lib/router.svelte'
  import { route } from '../lib/routes'
  import { handle_err } from '../lib/handle_err'
  import Loading from '../components/Loading.svelte'
  import ThemeSwitcher from '../components/ThemeSwitcher.svelte'
  import Icon from '../components/Icon.svelte'

  const { unlock, biometric, wallet } = root_store
  let loading = $state(true)
  let error = $state('')
  let disposed = false

  async function load() {
    loading = true
    error = ''

    try {
      const selected = unlock.target_wallet
      const wallets = await unlock.load_available_wallets()

      if (unlock.target_wallet && !wallets.includes(unlock.target_wallet))
        unlock.target_wallet = null
      if (unlock.target_wallet !== selected) unlock.set_passphrase('')
      if (disposed) return

      if (!wallets.length) {
        router.navigate(route.create_wallet, true)

        return
      }

      await biometric.load_for(wallets)

      if (disposed) return

      loading = false
      await autoPrompt()
    } catch (cause) {
      error = String(cause)
    } finally {
      loading = false
    }
  }

  function complete(chain: string) {
    if (disposed) return

    unlock.set_passphrase('')
    root_store.on_unlock()
    router.navigate(chain === 'Bitcoin' ? route.bitcoin : route.ethereum, true)
  }

  async function passwordUnlock() {
    if (!unlock.target_wallet || !unlock.passphrase || unlock.loader.loading)
      return

    try {
      complete(await unlock.unlock_wallet(wallet))
    } catch (cause) {
      error = String(cause)
    }
  }

  async function biometricUnlock() {
    const target = unlock.target_wallet

    if (!target || unlock.loader.loading) return

    unlock.loader.start()

    try {
      const result = await biometric.attempt_unlock(target)

      if (result && !disposed) {
        wallet.init(target, result)
        unlock.set_isunlocked(true)
        complete(result.last_used_chain)
      }
    } catch (cause) {
      error = String(cause)
    } finally {
      unlock.loader.stop()
    }
  }

  async function autoPrompt() {
    const target = unlock.target_wallet

    if (
      biometric.should_auto_prompt_for(
        target,
        !unlock.loader.loading && !unlock.is_unlocked,
      ) &&
      target
    ) {
      biometric.mark_auto_attempted(target)
      await biometricUnlock()
    }
  }

  function selectWallet(name: string) {
    unlock.set_target_wallet(name)
    error = ''
    autoPrompt().catch(handle_err)
  }

  function keyboard(event: KeyboardEvent) {
    const refresh =
      event.key === 'F5' ||
      ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'r')

    if (!refresh) return

    event.preventDefault()

    if (!loading && !unlock.loader.loading) load().catch(handle_err)
  }

  onMount(() => {
    wallet.reset()
    unlock.reset()
    biometric.reset()
    load().catch(handle_err)

    return () => {
      disposed = true
      unlock.set_passphrase('')
    }
  })
</script>

<svelte:window onkeydown={keyboard} />

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
        <form
          class="stack items-center"
          onsubmit={(event) => {
            event.preventDefault()
            passwordUnlock()
          }}
        >
          <label class="field">
            <span class="sr-only">Passphrase</span>
            <input
              class="input input-soft login-passphrase"
              type="password"
              placeholder="Passphrase"
              autocomplete="off"
              bind:value={unlock.passphrase}
              {@attach (element) => {
                element.focus()
              }}
            >
          </label>
          <button
            type="submit"
            class="sr-only"
            tabindex="-1"
            disabled={!unlock.passphrase}
          >
            Unlock wallet
          </button>
        </form>
      {/key}
      {#if biometric.is_enabled_for(unlock.target_wallet)}
        <button
          class="btn btn-sm preset-tonal-surface"
          onclick={biometricUnlock}
        >
          <Icon name="fingerprint" />
          Use Touch ID
        </button>
      {/if}
    {/if}
    {#if error}
      <p role="alert" class="error">{error}</p>
      <button class="btn btn-sm preset-tonal-surface" onclick={() => load()}>
        Reload wallets
      </button>
    {/if}
  </div>
  <nav class="login-wallets row" aria-label="Wallets">
    {#each unlock.available_wallets as name (name)}
      <button
        class={[
          'btn btn-sm',
          unlock.target_wallet === name
            ? 'preset-filled-surface-500'
            : 'preset-outlined-surface-500',
        ]}
        title={name}
        aria-pressed={unlock.target_wallet === name}
        disabled={loading || unlock.loader.loading}
        onclick={() => selectWallet(name)}
      >
        {name}
      </button>
    {/each}
    <button
      class="btn-icon"
      aria-label="Add a wallet"
      title="Add a wallet"
      disabled={loading || unlock.loader.loading}
      onclick={() => router.navigate(route.create_wallet)}
    >
      <Icon name="plus" />
    </button>
    <ThemeSwitcher iconOnly />
  </nav>
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

  .login-passphrase {
    width: 200px;
  }

  .login-wallets {
    position: fixed;
    inset-inline: 0;
    bottom: 16px;
    justify-content: center;
    padding-inline: 16px;
    max-height: 35dvh;
    overflow-y: auto;
  }

  .login-wallets .btn {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
  }

  @media (max-height: 450px) {
    .login-content {
      padding-top: 16px;
    }
  }
</style>
