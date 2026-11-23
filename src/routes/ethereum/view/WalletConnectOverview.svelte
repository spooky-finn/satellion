<script lang="ts">
  import Copyable from '../../../components/Copyable.svelte'
  import type { WalletConnectVM } from '../view_model/walletconnect.svelte'

  let { vm }: { vm: WalletConnectVM } = $props()

  const busy = $derived(vm.busy || vm.busy_approving)
  const pairing_disabled = $derived(busy || vm.waiting)

  function connect(event: SubmitEvent) {
    event.preventDefault()
    void vm.connect()
  }
</script>

{#if vm.settings?.disabled_reason}
  <p class="notice">{vm.settings.disabled_reason}</p>
{:else if vm.settings}
  {#if vm.result}
    <div class="soft-card row between" role="status">
      <strong>Transaction submitted</strong>
      <Copyable value={vm.result} label="Copy transaction hash" />
    </div>
  {/if}

  <form class="stack compact" onsubmit={connect}>
    <label class="field">
      WalletConnect link from the dApp QR code
      <input
        class="input"
        type="password"
        autocomplete="off"
        spellcheck="false"
        placeholder="wc:…"
        bind:value={vm.uri}
        disabled={pairing_disabled}
        required
      >
    </label>
    <div class="row between">
      <p role="status">
        {#if vm.waiting}
          Waiting for dApp…
        {:else if vm.busy}
          Connecting…
        {:else if vm.busy_approving}
          Processing request…
        {/if}
      </p>
      <button
        type="submit"
        class="btn preset-filled-primary-500"
        disabled={pairing_disabled || !vm.uri.trim()}
      >
        Connect
      </button>
    </div>
  </form>

  {#if vm.sessions.length}
    <section class="stack compact">
      <h3>Connected dApps</h3>
      <ul class="stack compact">
        {#each vm.sessions as app (app.topic)}
          <li class="soft-card row between">
            <div class="app-description">
              <strong>{app.name}</strong>
              <div>{app.origin}</div>
              <div class="muted">
                {app.chains.map(({ id }) => vm.chain_name(id)).join(', ')}
              </div>
            </div>
            <button
              type="button"
              class="btn btn-plain"
              aria-label={`Disconnect ${app.name} (${app.origin})`}
              onclick={() => vm.disconnect(app.topic)}
            >
              Disconnect
            </button>
          </li>
        {/each}
      </ul>
    </section>
  {/if}
{:else if vm.error}
  <button
    type="button"
    class="btn preset-tonal-primary"
    onclick={() => vm.show(vm.address)}
  >
    Retry
  </button>
{:else}
  <p role="status">Loading WalletConnect settings…</p>
{/if}

<style>
  .app-description {
    flex: 1 1 12rem;
    min-width: 0;
    overflow-wrap: anywhere;
  }
</style>
