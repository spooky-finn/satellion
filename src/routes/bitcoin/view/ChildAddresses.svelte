<script lang="ts">
  import { DeriveChildVM } from '../view_model/derive_child.vm.svelte'
  import { handle_err } from '../../../lib/handle_err'
  import Modal from '../../../components/Modal.svelte'
  import Copyable from '../../../components/Copyable.svelte'
  import Loading from '../../../components/Loading.svelte'
  import DeriveAddress from './DeriveAddress.svelte'
  import type { BitcoinWalletVM } from '../view_model/wallet.vm.svelte'

  let { btc }: { btc: BitcoinWalletVM } = $props()

  const derive = new DeriveChildVM()

  async function openDerive() {
    derive.set_is_open(true)
    await derive.next_unused_key_index()
  }

  async function deriveAddress() {
    await derive.derive()
    await btc.child_list.fetch()
  }
</script>

<Modal
  open={btc.child_list.is_open}
  title="Child addresses"
  fullscreen
  onclose={() => btc.child_list.set_open(false)}
>
  <div class="row">
    <button
      class="btn preset-filled-primary-500"
      onclick={(event) => {
        event.currentTarget.focus()
        openDerive().catch(handle_err)
      }}
    >
      Derive
    </button>
    <button
      class="btn btn-plain"
      onclick={() => btc.child_list.fetch().catch(handle_err)}
    >
      Refresh
    </button>
  </div>
  {#if btc.child_list.loader.loading}
    <Loading label="Loading addresses…" />
  {/if}
  {#if btc.child_list.addresses.length}
    <div class="table-wrap">
      <table class="table">
        <thead>
          <tr>
            <th>Label</th>
            <th>Derivation path</th>
            <th>Address</th>
          </tr>
        </thead>
        <tbody>
          {#each btc.child_list.addresses as address (address.path)}
            <tr>
              <td>{address.label}</td>
              <td class="mono text-xs">{address.path}</td>
              <td>
                <Copyable value={address.address} label="Copy child address" />
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <p>No addresses derived yet.</p>
  {/if}
  <DeriveAddress {derive} onderive={deriveAddress} />
</Modal>
