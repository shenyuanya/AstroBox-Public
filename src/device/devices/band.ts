import Device from "../device";
import xiaomiBandSVG from "./device.illustration/xiaomi-band.svg";
import xiaomiBandMiniSVG from "./illustration.s/xiaomi-band.svg";
export const xiaomiBand = {
    name: "Xiaomi Band series",
    img: xiaomiBandSVG,
    miniImg: xiaomiBandMiniSVG,
    nameRegex: /Xiaomi Smart Band \w\w? ?\S{4}?|小米手环\w\w?/i
} satisfies Device;