<script lang="ts">
  import { notifier } from '../lib/notifier'
</script>

<div
  class="fixed right-4 bottom-4 z-[100] flex max-w-[calc(100vw-2rem)] flex-col gap-2"
  aria-live="polite"
>
  {#each notifier.list() as notification (notification.id)}
    <div
      class={[
        'card max-w-md px-4 py-3 shadow-lg break-words',
        notification.level === 'err'
          ? 'preset-filled-error-500'
          : 'preset-filled-success-500',
      ]}
      role={notification.level === 'err' ? 'alert' : 'status'}
    >
      {notification.msg}
    </div>
  {/each}
</div>
