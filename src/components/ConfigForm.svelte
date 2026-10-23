<script lang="ts">
  import { Switch } from '@skeletonlabs/skeleton-svelte'
  import ConfigForm from './ConfigForm.svelte'
  import {
    type FieldSchema,
    resolveSchema,
    primaryType,
    isNullable,
  } from '../lib/config_schema'

  let {
    root,
    schema,
    values,
    onchange,
    path = [],
  }: {
    root: FieldSchema
    schema: FieldSchema
    values: Record<string, unknown>
    onchange: (path: string[], value: unknown) => void
    path?: string[]
  } = $props()

  const resolved = $derived(resolveSchema(schema, root))
</script>

{#each Object.entries(resolved.properties ?? {}) as [ name, fieldSchema ] (name)}
  {@const field = resolveSchema(fieldSchema, root)}
  {@const type = primaryType(field)}
  {@const label = field.title ?? name.replaceAll('_', ' ')}
  {@const value = values[name]}
  {#if type === 'object' && field.properties}
    <fieldset class="config-section stack">
      <legend class="eyebrow">{label}</legend>
      <ConfigForm
        {root}
        schema={field}
        values={(value as Record<string, unknown>) ?? {}}
        {onchange}
        path={[...path, name]}
      />
      <hr>
    </fieldset>
  {:else if type === 'boolean'}
    {#if field.readOnly}
      {#if value}
        <div class="row between">
          <span>{label}</span
          ><span class="text-sm text-green-700 dark:text-green-400"
            >Active</span
          >
        </div>
      {/if}
    {:else}
      <div class="stack gap-1">
        <Switch
          class="flex w-full items-center justify-between gap-4"
          checked={!!value}
          onCheckedChange={(event) => onchange([...path, name], event.checked)}
        >
          <Switch.Label>{label}</Switch.Label>
          <Switch.Control><Switch.Thumb /></Switch.Control>
          <Switch.HiddenInput />
        </Switch>
        {#if field.description}
          <p class="muted">{field.description}</p>
        {/if}
      </div>
    {/if}
  {:else if type === 'integer' || type === 'number'}
    <label class="row between">
      <span class="stack gap-0"
        ><span>{label}</span>
        {#if field.description}
          <span class="muted">{field.description}</span>
        {/if}</span
      >
      <input
        class="input input-sm config-number"
        type="number"
        value={(value as number) ?? 0}
        min={field.minimum}
        max={field.maximum}
        step={type === 'integer' ? 1 : 'any'}
        readonly={field.readOnly}
        oninput={(event) => {
          const number = event.currentTarget.valueAsNumber
          if (Number.isFinite(number)) onchange([...path, name], number)
        }}
      >
    </label>
  {:else}
    <label class="field"
      ><span class="config-label">{label}</span>
      <input
        class="input input-sm"
        value={(value as string) ?? ''}
        readonly={field.readOnly}
        oninput={(event) =>
          onchange(
            [...path, name],
            isNullable(field) && !event.currentTarget.value
              ? null
              : event.currentTarget.value,
          )}
      >
      {#if field.description}
        <span class="muted">{field.description}</span>
      {/if}</label
    >
  {/if}
{/each}

<style>
  .config-section {
    gap: 16px;
    min-width: 0;
    padding: 0;
    border: 0;
  }

  .config-section legend {
    float: left;
    width: 100%;
    padding: 0;
    margin-bottom: 0;
  }

  .config-section hr {
    margin: 0;
  }

  .config-number {
    width: 80px;
  }

  .config-label {
    color: var(--app-muted);
  }
</style>
