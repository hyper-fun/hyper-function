import hfn from "../core";
import { pack } from "msgpackr/pack";

hfn.init(
  pack({
    dev: true,
    hfn_config_path: "/Users/afei/Desktop/aefe/hfn.json",
  })
);

class HomeViewModule {
  show(ctx) {}
  hide() {}
}

const homeView = new HomeViewModule();
console.log(Object.getOwnPropertyNames(HomeViewModule.prototype));
