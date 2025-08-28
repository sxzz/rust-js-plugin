export default {
  name: 'test-js-plugin',
  load(id) {
    if (id === '/virtual.js') {
      return `console.log('Hello from /virtual.js');`
    }
  },
}
