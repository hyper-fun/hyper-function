interface HyperFunctionModule {
  name: string;
  props: string[];
  instance: any;
}

export class HyperFunctionPackage {
  name: string;
  modules: Record<string, HyperFunctionModule> = {};
  constructor(modules: any[], opts: { name?: string } = {}) {
    this.name = opts.name || "";

    for (const Mod of modules) {
      const instance = new Mod();
      const name = instance.name || Mod.name;
      const props = Object.getOwnPropertyNames(Mod.prototype);

      this.modules[name.toLowerCase()] = {
        name,
        props,
        instance,
      };
    }
  }
}
