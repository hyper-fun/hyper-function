import core from "../core";
import { Model, Schema } from "./model";
import msgpack from "./msgpack";

export class Context {
  constructor(
    public packageId: number,
    public socketId: string,
    public headers: Record<string, string>,
    public cookies: Record<string, string>,
    public data: Model,
    private opts: {
      moduleId?: number;
      hfnId?: number;
      schemas: Map<string, Schema>;
    }
  ) {
    const {} = opts;
  }
  render(state: Model) {
    return this.setState(state);
  }
  setState(state: Model) {
    // not state model
    if (!state.schema.moduleId) return;

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
