<script lang="ts">
  import Copyable from '../../../components/Copyable.svelte'
  import Modal from '../../../components/Modal.svelte'
  import type { WalletConnectVM } from '../view_model/walletconnect.svelte'
  import WalletConnectOverview from './WalletConnectOverview.svelte'
  import WalletConnectReview from './WalletConnectReview.svelte'

  let { vm }: { vm: WalletConnectVM } = $props()

  const request = $derived(vm.request)
  const proposal = $derived(vm.proposal)
  const peer = $derived(request?.peer ?? proposal?.peer)
  const busy = $derived(vm.busy || vm.busy_approving)
  const review_busy = $derived(request ? vm.busy_approving : vm.busy)
  const title = $derived(
    request
      ? (request.review?.title ?? 'Review dApp request')
      : proposal
        ? 'Connect dApp'
        : 'WalletConnect',
  )

  function decide(approved: boolean) {
    if (request) void vm.decide_request(approved)
    else if (proposal) void vm.decide_proposal(approved)
  }
</script>

<Modal
  open={vm.open}
  {title}
  {busy}
  wide={!!request}
  onclose={() => vm.close()}
>
  {#if vm.error}
    <p role="alert" class="error">{vm.error}</p>
  {/if}

  {#if peer}
    <div class="peer soft-card stack compact">
      <strong>{peer.name}</strong>
      <div class="peer-domain stack">
        <span>{peer.origin}</span>
        <span class={peer.verified ? 'muted' : 'unverified'}>
          {peer.verified ? 'Verified domain' : 'Domain unverified'}
        </span>
      </div>
      <div class="row">
        <span class="muted">Account</span>
        <Copyable
          value={proposal?.address ?? vm.address}
          label="Copy connected account"
        />
      </div>
    </div>
  {/if}

  {#if request}
    {#if request.review}
      <WalletConnectReview
        review={request.review}
        method={request.event.params.request.method}
        chain={request.chain}
        chain_name={id => vm.chain_name(id)}
      />
    {:else}
      <p role="status">Preparing request on {vm.chain_name(request.chain)}…</p>
    {/if}
    {#if vm.busy_approving}
      <p role="status">Processing request…</p>
    {/if}
  {:else if proposal}
    <section class="stack compact">
      <h3>Permissions by network</h3>
      <dl class="stack compact">
        {#each proposal.chains as grant (grant.id)}
          <div class="soft-card stack compact">
            <dt><strong>{vm.chain_name(grant.id)}</strong></dt>
            <dd>
              {#if grant.methods.length}
                <ul class="method-list">
                  {#each grant.methods as method (method)}
                    <li><code>{method}</code></li>
                  {/each}
                </ul>
              {:else}
                <span class="muted">No request methods</span>
              {/if}
            </dd>
          </div>
        {/each}
      </dl>
    </section>
    {#if vm.busy}
      <p role="status">Processing connection…</p>
    {/if}
  {:else}
    <WalletConnectOverview {vm} />
  {/if}
  {#if request || proposal}
    <div class="row actions">
      <button
        type="button"
        class="btn btn-plain"
        disabled={review_busy}
        onclick={() => decide(false)}
      >
        Reject
      </button>
      <button
        type="button"
        class="btn preset-filled-primary-500"
        disabled={review_busy || (!!request && !request.review)}
        onclick={() => decide(true)}
      >
        {request ? 'Review in system dialog' : 'Connect'}
      </button>
    </div>
  {/if}
</Modal>

<style>
  .peer,
  .method-list {
    overflow-wrap: anywhere;
  }

  .peer-domain {
    gap: 4px;
  }

  .unverified {
    color: var(--app-warning-text);
    font-size: 12px;
  }

  .method-list {
    padding: 0;
    list-style: none;
    font-size: 13px;
  }

  .actions {
    justify-content: flex-end;
  }
</style>
