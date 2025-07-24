import Device from "../device";
import redmiWatchSVG from "./device.illustration/redmi-watch.svg";
import redmiWatchMiniSVG from "./illustration.s/redmi-watch.svg";

export const redmiWatch =  {
    name: "REDMI Watch series",
    img: redmiWatchSVG,
    miniImg: redmiWatchMiniSVG,
    nameRegex: /Redmi Watch \w/i
}satisfies Device;