<script lang="ts">
  import { MIN_PASSPHRASE_LEN } from '../../../bindings'
  import RecoveryWords from './RecoveryWords.svelte'
  import PassphraseFields from './PassphraseFields.svelte'
  import type { Snippet } from 'svelte'
  import type { CreateFlow } from './create_flow.svelte'

  let { flow, home }: { flow: CreateFlow; home: Snippet } = $props()

  const m = $derived(flow.mnemonic)
</script>

<h1>{flow.mode === 'import' ? 'Import' : 'Imagine a passphrase'}</h1>
<form
  class="stack compact items-center wallet-form"
  onsubmit={(event) => {
    event.preventDefault()
    flow.save()
  }}
>
  <label class="field"
    ><span class="sr-only">Wallet name</span>
    <input
      class="input wallet-field"
      placeholder="Wallet name"
      autocomplete="off"
      bind:value={m.wallet_name}
      disabled={flow.busy}
      required
    ></label
  >
  {#if flow.mode === 'import'}
    <RecoveryWords {flow} />
  {:else}
    <p class="passphrase-help">
      Use at least {MIN_PASSPHRASE_LEN} characters. Think of your wallet
      passphrase as an extra lock on your wallet. Your recovery words are the
      key, and the passphrase is the secret code that makes the key work. Keep
      the passphrase stored separately from your recovery words.
    </p>
  {/if}
  <PassphraseFields {flow} />
  <button
    class="btn preset-filled-primary-500"
    disabled={flow.busy ||
      !m.wallet_name.trim() ||
      !m.passphrase_store.is_passphrase_matched ||
      (flow.mode === 'import' && !flow.importComplete)}
  >
    {flow.busy
      ? 'Saving…'
      : flow.mode === 'import'
        ? 'Import'
        : 'Save'}
  </button>
  {@render home()}
</form>

<style>
  .wallet-form {
    width: 100%;
  }

  .wallet-field {
    width: 200px;
  }

  .passphrase-help {
    max-width: 550px;
    font-size: 14px;
    text-align: left;
  }
</style>
