import { Context } from "./context";
import { Model } from "./model";

interface HyperFunctionModule {
  name: string;
  props: string[];
  instance: any;
}

interface HyperFunctionMiddleware {
  beforeHfn?(context: Context): void;
  afterHfn?(context: Context): void;
  beforeRpc?(context: Context): void;
  afterRpc?(context: Context): void;
  onSetState?(context: Context, state: Model): void;
}

export class Package {
  name: string;
  modules: Record<string, HyperFunctionModule> = {};
  middlewares: HyperFunctionMiddleware[] = [];
  hooks: {
    beforeHfnHooks: HyperFunctionMiddleware["beforeHfn"][];
    afterHfnHooks: HyperFunctionMiddleware["afterHfn"][];
    beforeRpcHooks: HyperFunctionMiddleware["beforeRpc"][];
    afterRpcHooks: HyperFunctionMiddleware["afterRpc"][];
    onSetStateHooks: HyperFunctionMiddleware["onSetState"][];
  } = {
    beforeHfnHooks: [],
    afterHfnHooks: [],
    beforeRpcHooks: [],
    afterRpcHooks: [],
    onSetStateHooks: [],
  };

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
  use(middleware: HyperFunctionMiddleware) {
    this.middlewares.push(middleware);
    if (middleware.beforeHfn) {
      this.hooks.beforeHfnHooks.push(middleware.beforeHfn);
    }
    if (middleware.afterHfn) {
      this.hooks.afterHfnHooks.push(middleware.afterHfn);
    }
    if (middleware.beforeRpc) {
      this.hooks.beforeRpcHooks.push(middleware.beforeRpc);
    }
    if (middleware.afterRpc) {
      this.hooks.afterRpcHooks.push(middleware.afterRpc);
    }
    if (middleware.onSetState) {
      this.hooks.onSetStateHooks.push(middleware.onSetState);
    }
  }
  async runBeforeHfnHooks(context: Context) {
    for (const hook of this.hooks.beforeHfnHooks) {
      await hook!(context);
    }
  }
  async runAfterHfnHooks(context: Context) {
    for (const hook of this.hooks.afterHfnHooks) {
      await hook!(context);
    }
  }
  async runBeforeRpcHooks(context: Context) {
    for (const hook of this.hooks.beforeRpcHooks) {
      await hook!(context);
    }
  }
  async runAfterRpcHooks(context: Context) {
    for (const hook of this.hooks.afterRpcHooks) {
      await hook!(context);
    }
  }
  async runOnSetStateHooks(context: Context, state: Model) {
    for (const hook of this.hooks.onSetStateHooks) {
      await hook!(context, state);
    }
  }
}
