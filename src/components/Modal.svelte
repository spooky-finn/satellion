<script lang="ts">
  import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte'
  import { untrack, type Snippet } from 'svelte'
  import Icon from './Icon.svelte'

  let {
    open,
    title,
    onclose,
    children,
    wide = false,
    fullscreen = false,
    busy = false,
    nested = false,
  }: {
    open: boolean
    title: string
    onclose: () => void
    children: Snippet
    wide?: boolean
    fullscreen?: boolean
    busy?: boolean
    /** Keep nested dialogs inside their parent's focus scope. */
    nested?: boolean
  } = $props()

  let content: HTMLElement | undefined
  let opener: HTMLElement | undefined

  function attachContent(element: HTMLElement) {
    // Capture the opener once; changes in a nested dialog must not replace it.
    untrack(() => {
      opener =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : undefined
      content = element
      initialFocus()?.focus({ preventScroll: true })
    })

    const returnTarget = opener

    return () => {
      content = undefined
      requestAnimationFrame(() => {
        if (returnTarget?.isConnected)
          returnTarget.focus({ preventScroll: true })
      })
    }
  }

  function initialFocus() {
    const active = document.activeElement

    if (active instanceof HTMLElement && content?.contains(active))
      return active

    return (
      content?.querySelector<HTMLElement>(
        'input:not([type="hidden"]):not(:disabled), select:not(:disabled), textarea:not(:disabled), button:not(:disabled), [tabindex="0"]',
      ) ??
      content ??
      null
    )
  }
</script>

<Dialog
  {open}
  restoreFocus={false}
  closeOnEscape={!busy}
  closeOnInteractOutside={!busy}
  initialFocusEl={initialFocus}
  onOpenChange={(event) => {
    if (!event.open && !busy) onclose()
  }}
>
  {#if open}
    <Portal disabled={nested}>
      <Dialog.Backdrop class="modal-backdrop" />
      <Dialog.Positioner
        class={['modal-positioner', fullscreen && 'modal-fullscreen']}
      >
        <Dialog.Content
          class={['modal-content stack', wide && 'modal-wide']}
          onkeydown={(event) => {
            if (event.key !== 'Escape') return
            event.preventDefault()
            event.stopPropagation()
            if (!busy) onclose()
          }}
          {@attach attachContent}
        >
          <Dialog.Title class="modal-title" role="heading" aria-level={2}
            >{title}</Dialog.Title
          >
          {@render children()}
          <Dialog.CloseTrigger
            class="btn-icon modal-close"
            aria-label="Close dialog"
            disabled={busy}
            ><Icon name="close" /></Dialog.CloseTrigger
          >
        </Dialog.Content>
      </Dialog.Positioner>
    </Portal>
  {/if}
</Dialog>

<style>
  :global {
    .modal-backdrop {
      position: fixed;
      inset: 0;
      z-index: 40;
      background: rgb(0 0 0 / 0.25);
    }

    .modal-positioner {
      position: fixed;
      inset: 0;
      z-index: 50;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 16px;
    }

    .modal-content {
      position: relative;
      width: fit-content;
      min-width: 300px;
      max-width: min(520px, 100%);
      max-height: calc(100dvh - 32px);
      overflow-y: auto;
      padding: 24px 48px 24px 24px;
      border: 1px solid var(--app-border);
      border-radius: 12px;
      background: var(--app-bg);
      box-shadow: 0 8px 32px rgb(0 0 0 / 0.12);
    }

    .modal-wide {
      width: 900px;
      max-width: 100%;
    }

    .modal-title {
      color: var(--app-heading);
      font-size: 24px;
      line-height: 32px;
      font-weight: 600;
      letter-spacing: -0.025em;
    }

    .modal-close {
      position: absolute;
      top: 8px;
      right: 8px;
    }

    .modal-fullscreen {
      padding: 0;
    }

    .modal-fullscreen > .modal-content {
      width: 100%;
      max-width: 100%;
      height: 100%;
      max-height: 100dvh;
      padding: 16px 48px 16px 16px;
      border: 0;
      border-radius: 0;
      box-shadow: none;
      justify-content: flex-start;
    }

    .modal-fullscreen > .modal-content > .modal-title {
      color: var(--color-primary-500);
    }

    @media (max-width: 480px) {
      .modal-content {
        min-width: 0;
        width: 100%;
        padding: 20px;
        padding-top: 40px;
      }

      .modal-fullscreen > .modal-content {
        padding: 16px 40px 16px 16px;
      }
    }
  }
</style>
