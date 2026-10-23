<script lang="ts">
  import Icon from './Icon.svelte'
  import Copyable from './Copyable.svelte'

  let {
    height,
    hash,
    price,
  }: {
    height?: number | string | null
    hash?: string | null
    price?: number | string | null
  } = $props()
</script>

<div class="network-status row" aria-label="Network status">
  <Icon name="layers" size={16} />
  {#if height}
    <span>height = {height}</span>
  {/if}
  {#if hash}
    <span class="row hash"
      >hash = <Copyable value={hash} label="Copy block hash" small /></span
    >
  {/if}
  {#if price}
    <span>price = ${price}</span>
  {/if}
</div>

<style>
  .network-status {
    color: var(--app-muted);
    font-size: 12px;
    gap: 8px;
  }

  .network-status > :global(svg) {
    color: var(--app-secondary);
  }

  .hash {
    gap: 0;
  }
</style>
