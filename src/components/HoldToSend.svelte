<script lang="ts">
  import { onDestroy } from 'svelte'
  import { handle_err } from '../lib/handle_err'

  let {
    onsend,
    disabled = false,
    busy = false,
    holdMs = 3000,
  }: {
    onsend: () => Promise<unknown>
    disabled?: boolean
    busy?: boolean
    holdMs?: number
  } = $props()

  let holding = $state(false)
  let timer: ReturnType<typeof setTimeout> | undefined

  function cancel() {
    clearTimeout(timer)
    timer = undefined
    holding = false
  }

  /** Send only after a continuous hold; release, blur, and unmount cancel it. */
  function start() {
    if (disabled || busy || timer) return

    holding = true
    timer = setTimeout(() => {
      cancel()
      onsend().catch(handle_err)
    }, holdMs)
  }

  onDestroy(cancel)
</script>

<button
  class="btn preset-filled-primary-500 relative overflow-hidden select-none touch-none"
  disabled={disabled || busy}
  onpointerdown={start}
  onpointerup={cancel}
  onpointerleave={cancel}
  onpointercancel={cancel}
  onblur={cancel}
  onkeydown={(event) => {
    if (event.key === ' ' || event.key === 'Enter') {
      event.preventDefault()
      if (!event.repeat) start()
    }
  }}
  onkeyup={(event) => {
    if (event.key === ' ' || event.key === 'Enter') cancel()
  }}
>
  <span
    class="absolute inset-y-0 left-0 bg-white/25"
    style:width={holding ? '100%' : '0%'}
    style:transition={holding ? `width ${holdMs}ms linear` : 'none'}
  ></span>
  <span class="relative">{busy ? 'Sending…' : 'Hold to send'}</span>
</button>
