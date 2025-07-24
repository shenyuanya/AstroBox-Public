import useSWR, {SWRConfiguration} from 'swr';
import {invoke} from "@tauri-apps/api/core";

export default function useInvoke<T>(config?: SWRConfiguration) {
    return (...args: Parameters<typeof invoke>) => useSWR(args, (args) => invoke<T>(...args), config);
}

export function useIsSendingMass(config?: SWRConfiguration) {
    const {data: isSendingMass, mutate} = useSWR("miwear_is_sending_mass", invoke<boolean>, {
        dedupingInterval: 10,
        focusThrottleInterval: 0,
        fallbackData: false, ...config
    })
    return {isSendingMass, mutate}
}

export function useInvokeWithMass<T>(massSending: boolean, config?: SWRConfiguration) {
    return (...args: Parameters<typeof invoke>) => useSWR(massSending ? null : args, (args) => massSending ? undefined : invoke<T>(...args), config);
}