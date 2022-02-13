import core from "../core";
import { pack } from "msgpackr/pack";
import { unpack } from "msgpackr/unpack";

import { HyperFunctionPackage } from "./package";

interface RunOptions {
  dev: boolean;
  hfnConfigPath?: string;
}

export function run(
  packages: HyperFunctionPackage[],
  opts: RunOptions = { dev: false }
) {
  const mainPkg = packages.find((pkg) => pkg.name === "");
  if (!mainPkg) {
    throw new Error('"main" package is not found');
  }

  const pkgNames = packages.map((pkg) => pkg.name.toLowerCase());
  const resultBuffer = core.init(
    pack({
      dev: true,
      hfn_config_path: "/Users/afei/Desktop/aefe/hfn.json",
      pkg_names: pkgNames,
    })
  );

  console.log(unpack(resultBuffer));
}

run([
  new HyperFunctionPackage([
    class HomeView {
      show() {}
    },
  ]),
]);
