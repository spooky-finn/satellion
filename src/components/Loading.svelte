<script lang="ts">
  import { Progress } from '@skeletonlabs/skeleton-svelte'

  let { label = 'Loading…' }: { label?: string } = $props()
</script>

<Progress value={null} class="stack w-full">
  <Progress.Label class="muted">{label}</Progress.Label>
  <Progress.Track><Progress.Range /></Progress.Track>
</Progress>
