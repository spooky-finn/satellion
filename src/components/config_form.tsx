import { Divider, Input, Stack, Switch } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { type ReactNode } from 'react'
import { P, Row } from '../shortcuts'

export interface FieldSchema {
  type?: string
  title?: string
  description?: string
  readOnly?: boolean
  properties?: Record<string, FieldSchema>
  anyOf?: FieldSchema[]
  allOf?: FieldSchema[]
  $ref?: string
  minimum?: number
  maximum?: number
  definitions?: Record<string, FieldSchema>
  $defs?: Record<string, FieldSchema>
}

function resolveSchema(schema: FieldSchema, root: FieldSchema): FieldSchema {
  if (schema.$ref) {
    const key = schema.$ref.split('/').pop()!
    const def = root.definitions?.[key] ?? root.$defs?.[key]
    if (def) return resolveSchema(def, root)
  }
  if (schema.allOf) {
    const merged = schema.allOf.reduce<FieldSchema>(
      (acc, s) => ({ ...acc, ...resolveSchema(s, root) }),
      {},
    )
    const { allOf: _allOf, ...rest } = schema
    return { ...merged, ...rest }
  }
  return schema
}

function primaryType(schema: FieldSchema): string | null {
  if (schema.type && typeof schema.type === 'string') return schema.type
  if (schema.anyOf) {
    const nonNull = schema.anyOf.find(s => s.type !== 'null')
    return (nonNull?.type as string) ?? null
  }
  return null
}

const Section = ({
  title,
  children,
}: {
  title: string
  children: ReactNode
}) => (
  <Stack gap={2}>
    <P
      level="body-xs"
      color="neutral"
      sx={{ textTransform: 'uppercase', letterSpacing: '0.08em' }}
    >
      {title}
    </P>
    {children}
    <Divider />
  </Stack>
)

const SettingRow = ({
  label,
  description,
  children,
}: {
  label: string
  description?: string
  children: ReactNode
}) => (
  <Row alignItems="center" justifyContent="space-between">
    <Stack gap={0.25}>
      <P level="body-md">{label}</P>
      {description && (
        <P level="body-xs" color="neutral">
          {description}
        </P>
      )}
    </Stack>
    {children}
  </Row>
)

interface FieldProps {
  name: string
  schema: FieldSchema
  value: unknown
  root: FieldSchema
  onChangePath: (path: string[], value: unknown) => void
}

const ConfigField = observer(
  ({ name, schema, value, root, onChangePath }: FieldProps) => {
    const resolved = resolveSchema(schema, root)
    const label = resolved.title ?? name
    const type = primaryType(resolved)
    const nullable = resolved.anyOf?.some(s => s.type === 'null') ?? false

    if (type === 'object' && resolved.properties) {
      return (
        <Section title={label}>
          <DynamicConfigForm
            root={root}
            schema={resolved}
            values={(value as Record<string, unknown>) ?? {}}
            onChangePath={onChangePath}
          />
        </Section>
      )
    }

    if (resolved.readOnly && type === 'boolean') {
      if (!value) return null
      return (
        <SettingRow label={label} description={resolved.description}>
          <P level="body-sm" color="success">
            Active
          </P>
        </SettingRow>
      )
    }

    if (type === 'boolean') {
      return (
        <SettingRow label={label} description={resolved.description}>
          <Switch
            checked={(value as boolean) ?? false}
            onChange={e => onChangePath([], e.target.checked)}
          />
        </SettingRow>
      )
    }

    if (type === 'integer' || type === 'number') {
      return (
        <SettingRow label={label} description={resolved.description}>
          <Input
            type="number"
            value={(value as number) ?? 0}
            onChange={e => {
              const v = parseInt(e.target.value, 10)
              if (!Number.isNaN(v)) onChangePath([], v)
            }}
            slotProps={{
              input: { min: resolved.minimum, max: resolved.maximum },
            }}
            size="sm"
            sx={{ width: 80 }}
          />
        </SettingRow>
      )
    }

    return (
      <Stack gap={0.5}>
        <P level="body-sm" color="neutral">
          {label}
        </P>
        <Input
          value={(value as string | null) ?? ''}
          onChange={e =>
            onChangePath(
              [],
              nullable && !e.target.value ? null : e.target.value,
            )
          }
          size="sm"
        />
        {resolved.description && (
          <P level="body-xs" color="neutral">
            {resolved.description}
          </P>
        )}
      </Stack>
    )
  },
)

interface Props {
  root: FieldSchema
  schema: FieldSchema
  values: Record<string, unknown>
  onChangePath: (path: string[], value: unknown) => void
}

export const DynamicConfigForm = observer(
  ({ root, schema, values, onChangePath }: Props) => {
    const resolved = resolveSchema(schema, root)
    if (!resolved.properties) return null

    return (
      <>
        {Object.entries(resolved.properties).map(([key, fieldSchema]) => (
          <ConfigField
            key={key}
            name={key}
            schema={fieldSchema}
            value={values[key]}
            root={root}
            onChangePath={(subPath, v) => onChangePath([key, ...subPath], v)}
          />
        ))}
      </>
    )
  },
)
