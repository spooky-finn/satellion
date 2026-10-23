<script lang="ts">
  import { onDestroy } from 'svelte'
  import { root_store } from '../view_model/root.svelte'
  import { router } from '../lib/router.svelte'
  import { route } from '../lib/routes'
  import Loading from '../components/Loading.svelte'
  import Navbar from '../components/Navbar.svelte'
  import Icon from '../components/Icon.svelte'
  import { CreateFlow } from './wallet/create/create_flow.svelte'
  import ChooseMethod from './wallet/create/ChooseMethod.svelte'
  import GeneratePhrase from './wallet/create/GeneratePhrase.svelte'
  import VerifyPhrase from './wallet/create/VerifyPhrase.svelte'
  import WalletCredentials from './wallet/create/WalletCredentials.svelte'

  const flow = new CreateFlow(
    () => root_store.mnemonic_wordlist,
    () => router.navigate(route.unlock_wallet, true),
  )

  onDestroy(() => flow.clearSecrets())
</script>

<svelte:window
  onkeydown={(event) => {
    flow.capsLock = event.getModifierState('CapsLock')
    if (event.code.startsWith('Key'))
      flow.keyboardLang = navigator.language || 'en'
  }}
/>

{#snippet home()}
  <button
    class="btn preset-tonal-surface"
    type="button"
    aria-label="Back to wallets"
    title="Back to wallets"
    disabled={flow.busy}
    onclick={() => router.navigate(route.unlock_wallet)}
  >
    <Icon name="home" />
  </button>
{/snippet}

{#if flow.mode === 'verify'}
  <Navbar hideLedgers />
{/if}

<section class="create-page stack compact items-center">
  {#if flow.mode === 'choose'}
    <ChooseMethod {flow} {home} />
  {:else if flow.mode === 'generate'}
    <GeneratePhrase {flow} {home} />
  {:else if flow.mode === 'verify'}
    <VerifyPhrase {flow} {home} />
  {:else}
    <WalletCredentials {flow} {home} />
  {/if}

  {#if flow.busy}
    <Loading label="Preparing your wallet…" />
  {/if}
  {#if flow.error}
    <p role="alert" class="error">{flow.error}</p>
  {/if}
</section>

<datalist id="mnemonic-wordlist">
  {#each root_store.mnemonic_wordlist as word (word)}
    <option value={word}></option>
  {/each}
</datalist>

<style>
  .create-page {
    width: 100%;
    text-align: center;
  }
</style>
