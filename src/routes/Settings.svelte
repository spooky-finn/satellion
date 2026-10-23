<script lang="ts">
  import { onMount } from 'svelte'
  import { root_store } from '../view_model/root.svelte'
  import { router } from '../lib/router.svelte'
  import { route } from '../lib/routes'
  import ConfigForm from '../components/ConfigForm.svelte'
  import Loading from '../components/Loading.svelte'
  import Icon from '../components/Icon.svelte'
  import BiometricSetting from './settings/BiometricSetting.svelte'
  import WalletSettings from './settings/WalletSettings.svelte'
  import ForgetWallet from './settings/ForgetWallet.svelte'

  const { settings: s, wallet } = root_store
  let loading = $state(true)
  let saving = $state(false)
  let forgetting = $state(false)
  let error = $state('')

  async function load() {
    loading = true

    try {
      await s.load_settings()
    } catch (cause) {
      error = String(cause)
    } finally {
      loading = false
    }
  }

  async function save() {
    saving = true

    try {
      await s.save()
    } catch (cause) {
      error = String(cause)
    } finally {
      saving = false
    }
  }

  async function forget() {
    if (!wallet.name || saving) return

    saving = true

    try {
      await wallet.forget(wallet.name)
      wallet.reset()
      root_store.unlock.reset()
      router.navigate(route.unlock_wallet, true)
    } catch (cause) {
      error = String(cause)
    } finally {
      saving = false
    }
  }

  onMount(() => {
    load()
  })
</script>

<section class="settings-page stack">
  <div class="row">
    <button
      class="btn-icon settings-back"
      aria-label="Back"
      title="Back"
      onclick={() => router.back()}
    >
      <Icon name="back" />
    </button>
    <h1 class="settings-title">Settings</h1>
  </div>
  {#if loading}
    <Loading />
  {:else}
    <form
      class="settings-form stack"
      onsubmit={(event) => {
        event.preventDefault()
        save()
      }}
    >
      {#if s.biometric_supported}
        <BiometricSetting {s} />
      {/if}
      {#if s.config && s.schema}
        <ConfigForm
          root={s.schema}
          schema={s.schema}
          values={s.config as unknown as Record<string, unknown>}
          onchange={(path, value) => s.set_value(path, value)}
        />
      {/if}
      {#if s.restart_hint}
        <p class="notice" role="status">
          Restart the app to apply network changes.
        </p>
      {/if}
      <button
        class="btn preset-filled-primary-500 settings-save"
        disabled={saving}
      >
        {saving ? 'Saving…' : 'Save'}
      </button>
    </form>
    <WalletSettings {s} {wallet} onforget={() => { forgetting = true }} />
  {/if}
  {#if error}
    <p role="alert" class="error">{error}</p>
  {/if}
</section>

<ForgetWallet
  open={forgetting}
  busy={saving}
  name={wallet.name}
  onclose={() => { forgetting = false }}
  onforget={forget}
/>

<style>
  .settings-page {
    width: min(100%, 496px);
    padding: 8px;
    gap: 24px;
  }

  .settings-title {
    font-weight: 600;
    font-size: 18px;
    line-height: 24px;
    letter-spacing: 0;
  }

  .settings-back {
    width: 42px;
  }

  .settings-form {
    gap: 24px;
  }

  .settings-save {
    width: 100%;
  }
</style>
