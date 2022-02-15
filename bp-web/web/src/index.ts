export class Foo {
  constructor() {
    console.log('bar');
  }

  async run() {
    console.log('run');
  }
}

new Foo();
