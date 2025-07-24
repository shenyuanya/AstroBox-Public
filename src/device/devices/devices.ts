import { xiaomiBand } from "./band";
import { xiaomiBandPro } from "./bandpro";
import { redmiWatch } from "./rw";
import { xiaomiWatchS } from "./s";
export const devices = [xiaomiBandPro,xiaomiBand, xiaomiWatchS, redmiWatch]
export type Device = typeof devices[number];
export enum Devices {
    xiaomiBand = "xiaomiBand",
    xiaomiWatchS = "xiaomiWatchS",
    redmiWatch = "redmiWatch",
    xiaomiBandPro = "xiaomiBandPro",
}