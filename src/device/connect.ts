import logger from "@/log/logger";
import { sleep } from "@/tools/common";
import { invoke } from "@tauri-apps/api/core";

export default async function connect(address: string,name:string,authKey:string) {
    await tryTryBreak(invoke, "connect.failed")('miwear_connect', { addr: address , name })
    logger.info("链接成功")
    await tryTryBreak(invoke, "connect.handshakeFailed")('miwear_start_hello');
    await sleep(500)
    logger.info("握手成功")
    await tryTryBreak(invoke, "connect.authFailed")('miwear_start_auth', { authKey })
    logger.info("Auth成功")
    invoke("miwear_get_codename");
    logger.info("Get codename成功")
    return true
}
function tryTryBreak<T extends (...args: any[]) => any>(
    func: T,
    errormsg: string
): (...args: Parameters<T>) => Promise<ReturnType<T>> {
    return async (...args) => {
        try {
            return await func(...args);
        } catch (error) {
            logger.error(error);
            throw new Error(errormsg);
        }
    };
}