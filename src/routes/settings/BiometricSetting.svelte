<script lang="ts">
  import { Switch } from '@skeletonlabs/skeleton-svelte'
  import type { SettingsVM } from '../../view_model/settings.svelte'
  import { handle_err } from '../../lib/handle_err'
  import Icon from '../../components/Icon.svelte'

  let { s }: { s: SettingsVM } = $props()
</script>

<div class="row between">
  <div>
    <div class="row">
      <Icon name="fingerprint" />
      <p>Unlock with Touch ID</p>
    </div>
    <p class="muted">Store your passphrase in the macOS Keychain.</p>
  </div>
  <Switch
    checked={s.biometric_enabled}
    disabled={s.biometric_busy}
    onCheckedChange={(event) =>
      s.toggle_biometric(event.checked).catch(handle_err)}
    ><Switch.Control><Switch.Thumb /></Switch.Control>
    <Switch.HiddenInput aria-label="Unlock with Touch ID" /></Switch
  >
</div>
