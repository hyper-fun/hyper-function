import { pack } from "msgpackr/pack";
import { unpack, unpackMultiple } from "msgpackr/unpack";

const msgpack = {
  encode(data: any, multiple?: boolean) {
    if (multiple) {
      return Buffer.concat(data.map((d: any) => pack(d)));
    }

    return pack(data);
  },
  decode(data: any, multiple?: boolean) {
    if (multiple) {
      return unpackMultiple(data);
    }

    return unpack(data);
  },
};

export default msgpack;
