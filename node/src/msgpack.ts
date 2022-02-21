import { Packr } from "msgpackr/pack";
import { unpack, unpackMultiple } from "msgpackr/unpack";

const packer = new Packr({ variableMapSize: true, useRecords: false });

const msgpack = {
  encode(data: any, multiple?: boolean) {
    if (!multiple) return packer.encode(data);
    return Buffer.concat(data.map((item: any) => packer.encode(item)));
  },
  decode(data: any, multiple?: boolean) {
    if (!multiple) return unpack(data);
    return unpackMultiple(data);
  },
};

export default msgpack;
