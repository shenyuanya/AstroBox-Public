import logger from "@/log/logger";
import { checkPermissions as bleCheckPermissions } from "@mnlphlp/plugin-blec";
import { checkPermissions as geoCheckPermissions, requestPermissions as geoRequestPermissions } from "@tauri-apps/plugin-geolocation";
import { platform } from "@tauri-apps/plugin-os";


export async function CheckLocationPermissionWithAlert() {
    var plat = await platform();
    logger.info(`Running on platform: ${plat}`);
    if(plat === "ios" || plat === "android"){
        let permissions = await geoCheckPermissions();
        /*
        安卓走SPP 不需要该权限
        if (!(permissions.location === 'granted')){
            alert("由于安卓的系统限制，在某些设备和系统策略上，您必须同意AstroBox使用位置信息权限才能正常扫描蓝牙设备。");
        }
        */
    }
}

export async function CheckBlePermissionWithAlert() {
    let permission = await bleCheckPermissions();
    if (!permission){
        throw new Error("permission.ble.required");
    }
}

export async function RequestLocationPermission() {
    var plat = await platform();
    if(plat === "ios" || plat === "android"){
        await geoRequestPermissions(['location']);
    }
}