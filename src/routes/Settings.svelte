<script lang="ts">
  import { onMount } from 'svelte'
  import { Switch } from '@skeletonlabs/skeleton-svelte'
  import { root_store } from '../view_model/root.svelte'
  import { router } from '../lib/router.svelte'
  import { route } from '../lib/routes'
  import { handle_err } from '../lib/handle_err'
  import { notifier } from '../lib/notifier'
  import ConfigForm from '../components/ConfigForm.svelte'
  import Loading from '../components/Loading.svelte'
  import Modal from '../components/Modal.svelte'
  import Icon from '../components/Icon.svelte'

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
        <div class="row between">
          <div>
            <div class="row">
              <Icon name="fingerprint" />
              <p>Unlock with Touch ID</p>
            </div>
            <p class="muted">Store your passphrase in the macOS Keychain.</p>
          </div>
          <Switch
            checked={s.biometric_enabled}
            disabled={s.biometric_busy}
            onCheckedChange={(event) =>
              s.toggle_biometric(event.checked).catch(handle_err)}
            ><Switch.Control><Switch.Thumb /></Switch.Control>
            <Switch.HiddenInput aria-label="Unlock with Touch ID" /></Switch
          >
        </div>
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
    <div class="stack">
      <p class="eyebrow">Wallet</p>
      <form
        class="row items-end"
        onsubmit={(event) => {
          event.preventDefault()
          s.rename_wallet()
            .then((ok) => {
              if (ok) notifier.ok('Wallet renamed')
            })
            .catch(handle_err)
        }}
      >
        <label class="field flex-1"
          >Wallet name<input
            class="input input-sm"
            bind:value={s.rename_draft}
            required
          ></label
        ><button
          class="btn preset-filled-primary-500"
          disabled={!s.rename_draft.trim() ||
            s.rename_draft.trim() === wallet.name}
        >
          Rename
        </button>
      </form>
      <div class="row between forget-row">
        <div class="forget-copy">
          <p>Forget wallet</p>
          <p class="muted">
            Remove this wallet from the device. Your funds are safe as long as
            you have the seed phrase.
          </p>
        </div>
        <button
          class="btn preset-tonal-error"
          onclick={(event) => {
            event.currentTarget.focus()
            forgetting = true
          }}
        >
          Forget
        </button>
      </div>
    </div>
  {/if}
  {#if error}
    <p role="alert" class="error">{error}</p>
  {/if}
</section>
<Modal
  open={forgetting}
  title="Forget wallet?"
  busy={saving}
  onclose={() => {
    forgetting = false
  }}
>
  <p>
    Remove {wallet.name} from this device? You need your recovery phrase and
    passphrase to restore access.
  </p>
  <div class="row">
    <button
      class="btn preset-tonal-surface"
      disabled={saving}
      onclick={() => {
        forgetting = false
      }}
    >
      Cancel
    </button><button
      class="btn preset-filled-error-500"
      disabled={saving}
      onclick={forget}
    >
      Forget wallet
    </button>
  </div>
</Modal>

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

  .forget-row {
    flex-wrap: nowrap;
  }

  .forget-copy {
    flex: 1;
  }

  .settings-save {
    width: 100%;
  }
</style>
