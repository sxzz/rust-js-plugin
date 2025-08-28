import { num } from './mod.js'
import * as ns from 'native-api'
import fs from 'node:fs'

export default {
  name: 'plugin',
  hook(param) {
    console.log('console log message')
    console.log('llrt version:', process.version)
    console.log('globalThis:', Object.keys(globalThis).join(', '))

    console.log('builtin_str:', ns.builtin_str)
    // console.log('echo:', ns.echo(10, 'ok', globalThis))
    console.log('add:', ns.add(0.1, 0.2))
    console.log('fs file length:', fs.readFileSync('./js/mod.js').length)

    if (process.env.BENCH) {
      benchmark()
    }

    return param * num
  },
}

function benchmark() {
  const times = 100000

  {
    const t = performance.now()
    for (let i = 0; i < times; i++) {
      ns.add(0.1, 0.2)
    }
    console.log('native add ops:', times / (performance.now() - t))
  }

  {
    const t = performance.now()
    for (let i = 0; i < times; i++) {
      add(0.1, 0.2)
    }
    console.log('js add ops:', times / (performance.now() - t))
  }
}

function add(...args) {
  return args.reduce((a, b) => a + b, 0)
}
