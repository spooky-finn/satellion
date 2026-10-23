export interface FieldSchema {
  type?: string | string[]
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

/** Resolve backend schema references and composed fields for the settings form. */
export function resolveSchema(
  schema: FieldSchema,
  root: FieldSchema,
): FieldSchema {
  if (schema.$ref) {
    const key = schema.$ref.split('/').pop()!
    const definition = root.definitions?.[key] ?? root.$defs?.[key]

    if (definition)
      return { ...resolveSchema(definition, root), ...schema, $ref: undefined }
  }

  if (schema.allOf) {
    const { allOf, ...rest } = schema

    return {
      ...Object.assign({}, ...allOf.map(field => resolveSchema(field, root))),
      ...rest,
    }
  }

  return schema
}

export function primaryType(schema: FieldSchema): string | undefined {
  if (Array.isArray(schema.type))
    return schema.type.find(type => type !== 'null')

  return (
    schema.type ??
    (schema.anyOf?.find(field => field.type !== 'null')?.type as
      | string
      | undefined)
  )
}

export function isNullable(schema: FieldSchema): boolean {
  return (
    !!(Array.isArray(schema.type) && schema.type.includes('null')) ||
    !!schema.anyOf?.some(field => field.type === 'null')
  )
}
