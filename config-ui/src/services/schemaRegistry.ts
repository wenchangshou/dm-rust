const schemaModules = import.meta.glob('/src/schemas/*.json', {
  eager: true,
  import: 'default'
}) as Record<string, Record<string, unknown>>

const schemas: Record<string, Record<string, unknown>> = {}

for (const path of Object.keys(schemaModules)) {
  const match = path.match(/\/([^/]+)\.json$/)
  const schema = schemaModules[path]
  if (match && match[1] && schema) {
    schemas[match[1]] = schema
  }
}

const protocolList = Object.keys(schemas).sort()

export function useSchemaRegistry() {
  return {
    schemas,
    protocolList
  }
}
