<script lang="ts">
  import { MIN_PASSPHRASE_LEN } from '../../../bindings'
  import type { CreateFlow } from './create_flow.svelte'

  let { flow }: { flow: CreateFlow } = $props()

  const m = $derived(flow.mnemonic)
</script>

<label class="field"
  ><span class="sr-only">Passphrase</span>
  <input
    class="input wallet-field"
    type="password"
    placeholder="Passphrase"
    autocomplete="new-password"
    bind:value={m.passphrase_store.passphrase}
    minlength={MIN_PASSPHRASE_LEN}
    disabled={flow.busy}
    required
  ></label
>
<div class="row muted">
  <span class={flow.capsLock ? 'caps-warning' : ''}
    >Caps Lock: {flow.capsLock ? 'ON' : 'OFF'}</span
  ><span>Language: {flow.keyboardLang || 'unknown'}</span>
</div>
<label class="field"
  ><span class="sr-only">Repeat passphrase</span>
  <input
    class="input wallet-field"
    type="password"
    placeholder="Repeat passphrase"
    autocomplete="new-password"
    bind:value={m.passphrase_store.repeat_passphrase}
    disabled={flow.busy}
    required
  ></label
>
{#if m.passphrase_store.is_mismatch}
  <p class="error" role="alert">
    Passphrases must match and meet the minimum length.
  </p>
{/if}

<style>
  .wallet-field {
    width: 200px;
  }

  .caps-warning {
    color: var(--app-warning-text);
  }
</style>
