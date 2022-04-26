import core from "../core";
import { Model, Schema } from "./model";
import msgpack from "./msgpack";
import { Package } from "./package";

export class Context {
  public data: Record<string, any> = {};
  constructor(
    public packageId: number,
    public pkg: Package,
    public socketId: string,
    public headers: Record<string, string>,
    public cookies: Record<string, string>,
    public body: Model,
    private opts: {
      moduleId?: number;
      hfnId?: number;
      schemas: Map<string, Schema>;
    }
  ) {
    const {} = opts;
  }
  async render(state: Model) {
    return await this.setState(state);
  }
  async setState(state: Model) {
    // not state model
    if (!state.schema.moduleId) return;

    await this.pkg.runOnSetStateHooks(this, state);
    const payload = msgpack.encode(
      [2, state.schema.pkgId, state.schema.moduleId, state.encode()],
      true
    );
    core.sendMessage(
      this.socketId,
      msgpack.encode([0, this.packageId, {}, payload], true)
    );
  }
  model(name: string) {
    const schema = this.opts.schemas.get(name);
    if (!schema) return;

    const model = new Model(schema, this.opts.schemas);

    return model;
  }
  setCookie(name: string, value: string, maxAge: number = 0, isPrivate = true) {
    const payload = msgpack.encode([3, name, value, maxAge, isPrivate], true);
    core.sendMessage(
      this.socketId,
      msgpack.encode([0, this.packageId, {}, payload], true)
    );
  }
  rpc() {}
  response() {}
}
