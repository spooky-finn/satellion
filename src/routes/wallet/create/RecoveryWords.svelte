<script lang="ts">
  import type { CreateFlow } from './create_flow.svelte'

  let { flow }: { flow: CreateFlow } = $props()
</script>

<div class="soft-card stack compact mnemonic-card">
  <div class="row between">
    <p class="muted">Enter your mnemonic phrase</p>
    <label class="field"
      ><span class="sr-only">Recovery phrase length</span
      ><select
        class="select select-sm"
        value={flow.wordCount}
        disabled={flow.busy}
        onchange={(event) => {
          flow.wordCount = Number(event.currentTarget.value)
          flow.words = Array(flow.wordCount).fill('')
        }}
      >
        <option value={12}>12 words</option>
        <option value={24}>24 words</option>
      </select></label
    >
  </div>
  <div class="word-inputs">
    {#each flow.words as _, index (index)}
      <label class="word-input row"
        ><span class="word-number" aria-hidden="true">{index + 1}</span
        ><span class="sr-only">Word {index + 1}</span>
        <input
          class="input input-sm"
          aria-label={`Word ${index + 1}`}
          type={flow.selectedWord === index ? 'text' : 'password'}
          value={flow.words[index]}
          autocomplete="off"
          spellcheck="false"
          disabled={flow.busy}
          list={flow.selectedWord === index
            ? 'mnemonic-wordlist'
            : undefined}
          onfocus={() => {
            flow.selectedWord = index
          }}
          onpaste={(event) => flow.paste(event)}
          oninput={(event) => {
            flow.words[index] = event.currentTarget.value
              .trim()
              .toLowerCase()
          }}
        >
      </label>
    {/each}
  </div>
</div>

<style>
  .word-inputs {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 4px;
    padding-block: 8px;
  }

  .mnemonic-card {
    max-width: min(100%, 900px);
  }

  .word-input {
    flex-wrap: nowrap;
    gap: 8px;
  }

  .word-number {
    width: 10px;
    text-align: right;
    font-size: 12px;
    color: var(--app-muted);
  }

  .word-input .input {
    width: 150px;
  }

  @media (max-width: 380px) {
    .word-input .input {
      width: 125px;
    }
  }
</style>
