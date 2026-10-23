<script lang="ts">
  import Modal from '../../components/Modal.svelte'

  let {
    open,
    busy,
    name,
    onclose,
    onforget,
  }: {
    open: boolean
    busy: boolean
    name?: string
    onclose: () => void
    onforget: () => Promise<void>
  } = $props()
</script>

<Modal {open} title="Forget wallet?" {busy} {onclose}>
  <p>
    Remove {name} from this device? You need your recovery phrase and passphrase
    to restore access.
  </p>
  <div class="row">
    <button class="btn preset-tonal-surface" disabled={busy} onclick={onclose}>
      Cancel
    </button><button
      class="btn preset-filled-error-500"
      disabled={busy}
      onclick={onforget}
    >
      Forget wallet
    </button>
  </div>
</Modal>
