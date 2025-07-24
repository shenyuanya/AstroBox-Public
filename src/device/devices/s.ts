import Device from "../device";
import xiaomiWatchSVG from "./device.illustration/xiaomi-watch.svg";
import xiaomiWatchMiniSVG from "./illustration.s/xiaomi-watch.svg";
export const xiaomiWatchS = {
    name: "Xiaomi Watch S series",
    img: xiaomiWatchSVG,
    miniImg: xiaomiWatchMiniSVG,
    nameRegex: /Xiaomi Watch S\w (eSIM )?\S{4}/i
} satisfies Device;