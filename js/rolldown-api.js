import { build } from 'rolldown'

const _1k_entry = Array.from(
  { length: 1000 },
  (_, i) => `export * from '/virtual-${i}';`,
).join('\n')

const plugin = {
  name: 'test-js-plugin',
  resolveId(id) {
    return id + '.js'
  },
  load(id) {
    if (id === '/entry.js') {
      return _1k_entry
    }
    return `export const _${id
      .replace('/virtual-', '')
      .replace('.js', '')} = 42`
  },
}

await build({
  input: ['/entry'],
  plugins: [plugin],
  write: false,
})
