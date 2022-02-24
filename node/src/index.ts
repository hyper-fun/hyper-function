import core from "../core";

import pkg from "../package.json";
import { HyperFunctionPackage } from "./package";
import { Model, Schema } from "./model";
import msgpack from "./msgpack";
import { Context } from "./context";

interface RunOptions {
  dev: boolean;
  hfnConfigPath?: string;
}

interface InitResult {
  upstream_id: string;
  packages: {
    id: number;
    name: string;
    full_name: string;
  }[];
  modules: {
    id: number;
    name: string;
    package_id: number;
  }[];
  models: {
    id: number;
    name: string;
    schema_id: number;
    package_id: number;
    module_id: number;
  }[];
  hfns: {
    id: number;
    name: string;
    schema_id: number;
    package_id: number;
    module_id: number;
  }[];
  rpcs: {
    id: number;
    name: string;
    req_schema_id: number;
    res_schema_id: number;
    package_id: number;
  }[];
  schemas: {
    id: number;
    package_id: number;
  }[];
  fields: {
    id: number;
    name: string;
    t: string;
    is_array: boolean;
    package_id: number;
    schema_id: number;
  }[];
}

export function run(
  packages: HyperFunctionPackage[],
  opts: RunOptions = { dev: false }
) {
  const mainPkg = packages.find((pkg) => pkg.name === "");
  if (!mainPkg) {
    throw new Error('"main" package is not found');
  }

  const pkgNames = packages.map((pkg) => pkg.name);
  const result: InitResult = msgpack.decode(
    core.init(
      msgpack.encode({
        dev: true,
        addr: "[::1]:3000",
        sdk: "node-" + pkg.version,
        hfn_config_path: "/Users/afei/Desktop/aefe/hfn.json",
        pkg_names: pkgNames,
      })
    )
  );

  const handlers = new Map<string, (ctx: Context) => void>();

  for (const pkgConfig of result.packages) {
    const pkg = packages.find((pkg) => pkg.name === pkgConfig.name);
    if (!pkg) continue;

    let modConfigs = result.modules.filter(
      (mod) => mod.package_id === pkgConfig.id
    );

    for (const modConfig of modConfigs) {
      const mod = pkg.modules[modConfig.name.toLowerCase()];
      if (!mod) continue;

      const hfnConfigs = result.hfns.filter(
        (hfn) =>
          hfn.module_id === modConfig.id && hfn.package_id === pkgConfig.id
      );

      for (const hfnConfig of hfnConfigs) {
        const hfnExists = mod.props.includes(hfnConfig.name);
        if (!hfnExists) continue;

        handlers.set(
          `${pkgConfig.id}-${modConfig.id}-${hfnConfig.id}`,
          mod.instance[hfnConfig.name]
        );
      }
    }
  }

  const schemas = new Map<string, Schema>();

  for (const schemaConfig of result.schemas) {
    const schema: Schema = {
      id: schemaConfig.id,
      pkgId: schemaConfig.package_id,
      fields: new Map(),
    };

    const fields = result.fields.filter(
      (field) =>
        field.schema_id === schema.id && field.package_id === schema.pkgId
    );

    for (const fieldConfig of fields) {
      const field = {
        id: fieldConfig.id,
        name: fieldConfig.name,
        type: fieldConfig.t,
        isArray: fieldConfig.is_array,
        pkgId: fieldConfig.package_id,
        schemaId: fieldConfig.schema_id,
      };

      schema.fields.set(field.name, field);
      schema.fields.set(field.id, field);
    }

    const model = result.models.find(
      (m) => m.schema_id === schema.id && m.package_id === schema.pkgId
    );

    if (model) {
      if (model.name === "") {
        schema.moduleId = model.module_id;
      }

      const pkg = result.packages.find((pkg) => pkg.id === model.package_id);
      const mod = result.modules.find(
        (m) => m.id === model.module_id && m.package_id === model.package_id
      );

      const key = `${pkg.id === 0 ? "" : pkg.name + "."}${mod.name}.${
        model.name || "State"
      }`;

      schemas.set(key, schema);
      schemas.set(`model-${pkg.id}-${mod.id}-${model.id}`, schema);
    }

    const hfn = result.hfns.find(
      (n) => n.schema_id === schema.id && n.package_id === schema.pkgId
    );

    if (hfn) {
      schema.hfnId = hfn.id;

      const pkg = result.packages.find((pkg) => pkg.id === hfn.package_id);
      const mod = result.modules.find(
        (m) => m.id === hfn.module_id && m.package_id === hfn.package_id
      );

      const key = `${pkg.id === 0 ? "" : pkg.name + "."}${mod.name}.${
        hfn.name
      }`;

      schemas.set(key, schema);
      schemas.set(`hfn-${pkg.id}-${mod.id}-${hfn.id}`, schema);
    }

    schemas.set(`${schema.pkgId}-${schema.id}`, schema);
  }

  core.run();

  (async () => {
    while (true) {
      const data = await core.read();
      const [pkgId, headers, payload, socketId] = msgpack.decode(
        data,
        true
      ) as any[];

      const msg = msgpack.decode(payload, true);
      switch (msg[0]) {
        case 1: {
          const [_, moduleId, hfnId, cookies, data] = msg;
          const id = `${pkgId}-${moduleId}-${hfnId}`;
          const schema = schemas.get(`hfn-${id}`);
          const dataModel = new Model(schema, schemas);
          if (data) dataModel.decode(data);

          const context = new Context(
            pkgId,
            socketId,
            headers,
            cookies,
            dataModel,
            {
              moduleId,
              hfnId,
              schemas,
            }
          );
          const handler = handlers.get(id);
          if (handler) handler(context);
        }
      }
    }
  })();
}

run([
  new HyperFunctionPackage([
    class HomeView {
      mount(ctx: Context) {
        console.log(ctx.headers);
        console.log(ctx.data.toObject());
        const state = ctx.model("homeView.State");
        state.set("str", "blabla!!❤️");
        state.set("strArr", ["1", "2", "4"]);
        state.set("int", 123);
        state.set("intArr", [123, 234, 456]);
        state.set("float", 1.2);
        state.set("floatArr", [2.3, 3.4, 5.5]);
        state.set("bool", true);
        state.set("boolArr", [true, false, true]);
        state.set("bytes", new Uint8Array([0xab]));
        state.set("bytesArr", [
          new Uint8Array([0xab]),
          new Uint8Array([0xcd]),
          new Uint8Array([0xef]),
        ]);
        const nested = ctx.model("homeView.ahaha");
        nested.set("id", 2323);
        nested.set("s", "baba");
        state.set("nested", nested);
        state.set("nestedArr", [nested, nested, nested]);

        ctx.render(state);
      }
    },
  ]),
]);
