<script lang="ts">
  import type { Snippet } from 'svelte'
  import type { CreateFlow } from './create_flow.svelte'

  let { flow, home }: { flow: CreateFlow; home: Snippet } = $props()
</script>

<h1>Add wallet</h1>
<div class="row flow-choice">
  <button
    class="btn preset-tonal-surface"
    disabled={flow.busy}
    onclick={() => {
      flow.mode = 'import'
    }}
  >
    Import
  </button>
  <button
    class="btn preset-tonal-surface"
    disabled={flow.busy}
    onclick={() => flow.generate()}
  >
    Generate
  </button>
</div>
{@render home()}

<style>
  .flow-choice {
    margin-block: 8px;
  }
</style>
