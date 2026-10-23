<script lang="ts">
  import { onDestroy } from 'svelte'
  import { notifier } from '../lib/notifier'

  let {
    value,
    label = 'Copy',
    n = 6,
    small = false,
  }: { value: string; label?: string; n?: number; small?: boolean } = $props()

  let copied = $state(false)
  let timer: ReturnType<typeof setTimeout> | undefined
  const display = $derived(
    value.length > n * 2 ? `${value.slice(0, n)}…${value.slice(-n)}` : value,
  )

  async function copy(event: MouseEvent) {
    event.stopPropagation()

    try {
      await navigator.clipboard.writeText(value)
      copied = true
      notifier.ok('Copied to clipboard')
      clearTimeout(timer)
      timer = setTimeout(() => (copied = false), 1500)
    } catch (error) {
      notifier.err(String(error))
    }
  }

  onDestroy(() => clearTimeout(timer))
</script>

{#if value}
  <button
    type="button"
    class={['copyable mono', small && 'copyable-small', copied && 'copied']}
    onclick={copy}
    aria-label={label}
    title={value}
  >
    {display}
  </button>
{/if}

<style>
  .copyable {
    display: inline;
    width: fit-content;
    max-width: 100%;
    padding: 2px 4px;
    border-radius: 6px;
    font: inherit;
    font-family:
      ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    white-space: nowrap;
    cursor: copy;
    text-align: left;
    color: inherit;
    transition: background-color 180ms ease;
  }

  .copyable:hover {
    background: var(--app-neutral-hover);
  }

  .copyable-small {
    font-size: 12px;
    line-height: 18px;
  }

  .copied {
    background: light-dark(#e3f3e8, #174126);
  }
</style>
