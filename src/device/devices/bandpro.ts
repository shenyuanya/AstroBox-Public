import Device from "../device";
import xiaomiBandProSVG from "./device.illustration/xiaomi-band-pro.svg";
import xiaomiBandProMiniSVG from "./illustration.s/xiaomi-band-pro.svg";
export const xiaomiBandPro = {
    name: "Xiaomi Band Pro series",
    img: xiaomiBandProSVG,
    miniImg: xiaomiBandProMiniSVG,
    nameRegex: /Xiaomi Smart Band \w\w? Pro .{4}|小米手环\w\w? Pro/i
} satisfies Device;