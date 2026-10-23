<script lang="ts">
  import type { Snippet } from 'svelte'
  import type { CreateFlow } from './create_flow.svelte'

  let { flow, home }: { flow: CreateFlow; home: Snippet } = $props()

  const m = $derived(flow.mnemonic)
</script>

<h1>Random secret private key</h1>
<button
  class="soft-card mnemonic-phrase"
  title="Copy recovery phrase"
  aria-label="Copy recovery phrase"
  onclick={() => flow.copyMnemonic()}
>
  {m.mnemonic.join(' ')}
</button>
{#if flow.copied}
  <p class="copied-text" role="status">Copied</p>
{/if}
<div class="row">
  {@render home()}
  <button
    class="btn btn-plain btn-neutral"
    disabled={flow.busy}
    onclick={() => flow.generate()}
  >
    Regenerate
  </button>
  <button
    class="btn preset-tonal-primary"
    disabled={flow.busy}
    onclick={() => {
      flow.mode = 'verify'
    }}
  >
    Continue
  </button>
</div>

<style>
  .mnemonic-phrase {
    max-width: min(100%, 900px);
    padding: 8px;
    font-size: 18px;
    line-height: 27px;
    text-align: left;
    cursor: copy;
  }

  .copied-text {
    color: light-dark(#1f7a40, #77c69a);
    font-size: 12px;
  }
</style>
