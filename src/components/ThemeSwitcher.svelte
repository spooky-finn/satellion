<script lang="ts">
  import { theme } from '../lib/theme.svelte'
  import Icon from './Icon.svelte'

  let { iconOnly = true }: { iconOnly?: boolean } = $props()
</script>

<button
  class={iconOnly ? 'btn-icon' : 'btn btn-plain btn-neutral'}
  aria-label="Toggle color theme"
  title={theme.dark ? 'Switch to light mode' : 'Switch to dark mode'}
  onclick={() => theme.toggle()}
>
  {#if iconOnly}
    <Icon name={theme.dark ? 'sun' : 'moon'} />
  {:else}
    {theme.dark ? 'Light' : 'Dark'}
  {/if}
</button>
