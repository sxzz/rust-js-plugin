import { num } from './mod.js'
import * as ns from 'native-api'

export default {
  name: 'plugin',
  hook(param) {
    console.log('console log message')
    console.log('globalThis: ' + Object.keys(globalThis).join(', '))

    console.log('builtin_str: ' + ns.builtin_str)
    // console.log('echo:', ns.echo(10, 'ok', globalThis))
    console.log('add: ' + ns.add(0.1, 0.2))

    let t = performance.now()
    const times = 100000
    for (let i = 0; i < times; i++) {
      ns.add(0.1, 0.2)
    }
    console.log('native add ops: ' + times / (performance.now() - t))

    t = performance.now()
    for (let i = 0; i < times; i++) {
      add(0.1, 0.2)
    }
    console.log('js add ops: ' + times / (performance.now() - t))

    return param * num
  },
}

function add(...args) {
  return args.reduce((a, b) => a + b, 0)
}
