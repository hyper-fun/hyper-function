import msgpack from "./msgpack";

export interface Schema {
  id: number;
  pkgId: number;
  moduleId?: number; // only module state model schema has this
  hfnId?: number; // only hfn schema has this
  fields: Map<
    string | number,
    {
      id: number;
      name: string;
      type: string;
      isArray: boolean;
      pkgId: number;
      schemaId: number;
    }
  >;
}

export class Model {
  private data: Record<string, any>;
  constructor(public schema: Schema, public schemas: Map<string, Schema>) {
    this.data = Object.create(null);
  }

  set(key: string, value: any) {
    if (typeof value === "undefined") return false;
    const field = this.schema.fields.get(key);
    if (!field) return false;

    const valueIsArray = Array.isArray(value);
    if (field.isArray !== valueIsArray) return false;

    if (field.type.length === 1) {
      // basic type
      if (valueIsArray) {
        for (let i = 0; i < value.length; i++) {
          if (!checkType(value[i], field.type)) return false;
        }
      } else {
        if (!checkType(value, field.type)) return false;
      }
    } else {
      // model type
      const targetSchema = this.schemas.get(field.type);
      if (!targetSchema) return;
      if (valueIsArray) {
        for (let i = 0; i < value.length; i++) {
          const item = value[i];
          if (
            !(item instanceof Model) ||
            item.schema.id !== targetSchema.id ||
            item.schema.pkgId !== targetSchema.pkgId
          ) {
            return;
          }
        }
      } else {
        if (
          !(value instanceof Model) ||
          value.schema.id !== targetSchema.id ||
          value.schema.pkgId !== targetSchema.pkgId
        ) {
          return;
        }
      }
    }

    this.data[key] = value;
    return true;
  }
  get(key: string) {
    return this.data[key];
  }
  has(key: string) {
    return !!this.data[key];
  }
  keys() {
    return Object.keys(this.data);
  }
  delete(key: string) {
    delete this.data[key];
  }
  encode() {
    const keys = this.keys();
    const dataArr = [];
    for (let i = 0; i < keys.length; i++) {
      const key = keys[i];
      const field = this.schema.fields.get(key)!;
      dataArr.push(field.id);

      let value;
      if (field.type.length === 1) {
        // scalar type
        value = this.data[key];
      } else {
        // model type
        if (field.isArray) {
          value = this.data[key].map((item: Model) => item.encode());
        } else {
          value = this.data[key].encode();
        }
      }

      dataArr.push(value);
    }

    return msgpack.encode(dataArr, true);
  }
  decode(data: Uint8Array) {
    if (!data.byteLength) return;
    let dataArr = [];
    try {
      dataArr = msgpack.decode(data, true);
    } catch (error) {
      console.log(error);
      return;
    }

    let field = null;
    for (let i = 0; i < dataArr.length; i++) {
      const item = dataArr[i];
      if (!field) {
        field = this.schema.fields.get(item);
        if (!field) return;
        continue;
      }

      let value;
      if (field.type.length === 1) {
        value = item;
      } else {
        const targetSchema = this.schemas.get(field.type);
        if (!targetSchema) return;
        if (field.isArray) {
          value = item.map((data: Uint8Array) => {
            const m = new Model(targetSchema, this.schemas);
            m.decode(data);
            return m;
          });
        } else {
          value = new Model(targetSchema, this.schemas);
          value.decode(item);
        }
      }

      this.set(field.name, value);
      field = null;
    }
  }
  fromObject(obj: any) {
    if (typeof obj !== "object") return null;
    Object.keys(obj).forEach((key) => {
      const field = this.schema.fields.get(key);
      if (!field) return null;
      if (field.type.length === 1) {
        this.set(key, obj[key]);
      } else {
        if (field.isArray) {
          this.set(
            key,
            obj[key].map((item: any) => {
              const m = new Model(this.schemas.get(field.type)!, this.schemas);
              m.fromObject(item);
              return m;
            })
          );
        } else {
          const m = new Model(this.schemas.get(field.type)!, this.schemas);
          m.fromObject(obj[key]);
          this.set(key, m);
        }
      }
    });

    return this;
  }
  toObject() {
    const obj: Record<string, any> = {};
    this.keys().forEach((key) => {
      const field = this.schema.fields.get(key);
      if (!field) return;
      if (field.type.length === 1) {
        obj[key] = this.get(key);
      } else {
        if (field.isArray) {
          obj[key] = this.get(key).map((item: Model) => item.toObject());
        } else {
          obj[key] = this.get(key).toObject();
        }
      }
    });
    return obj;
  }
}

function checkType(value: any, type: string): boolean {
  switch (type) {
    case "s":
      return value === value + "";
    case "i":
      return (
        Number.isInteger(value) && value <= 2147483647 && value >= -2147483648
      );
    case "f":
      return !isNaN(value) && typeof value === "number";
    case "b":
      return value === !!value;
    case "t":
      return value instanceof Uint8Array;
    default:
      return false;
  }
}
