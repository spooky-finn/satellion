<script lang="ts">
  import type { Snippet } from 'svelte'
  import type { CreateFlow } from './create_flow.svelte'

  let { flow, home }: { flow: CreateFlow; home: Snippet } = $props()

  const m = $derived(flow.mnemonic)
</script>

<h1>Enter your mnemonic words</h1>
<form
  class="stack compact items-center"
  onsubmit={(event) => {
    event.preventDefault()
    if (m.verify()) flow.mode = 'passphrase'
  }}
>
  {#each m.verification_indices as index (index)}
    <label class="word-input row"
      ><span class="word-number" aria-hidden="true">{index + 1}</span
      ><span class="sr-only">Word {index + 1}</span>
      <input
        class="input input-sm"
        aria-label={`Word ${index + 1}`}
        autocomplete="off"
        spellcheck="false"
        list="mnemonic-wordlist"
        value={m.verification_words[index] ?? ''}
        oninput={(event) =>
          m.set_verification_words(
            index,
            event.currentTarget.value.toLowerCase(),
          )}
      >
    </label>
  {/each}
  <button class="btn preset-tonal-primary">Verify</button>
  {@render home()}
</form>
{#if m.verification_successfull === false}
  <p class="error" role="alert">
    Verification failed. Check your saved phrase.
  </p>
{/if}

<style>
  .word-input {
    flex-wrap: nowrap;
    gap: 8px;
  }

  .word-number {
    width: 10px;
    text-align: right;
    font-size: 12px;
    color: var(--app-muted);
  }

  .word-input .input {
    width: 150px;
  }

  @media (max-width: 380px) {
    .word-input .input {
      width: 125px;
    }
  }
</style>
