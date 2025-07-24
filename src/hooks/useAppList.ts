import useSWRInfinite from 'swr/infinite'
import {providerManager} from "@/community/manager";

const SPLIT = "__split__"
export default function useAppList(provider?: string, search?: string, limit: number = 10) {
    return useSWRInfinite((index) => {
        if (!provider) return null
        if (!search) search = ""
        return provider + SPLIT + index + SPLIT + search + SPLIT + limit;
    }, getPage, {
        revalidateFirstPage: false,
        revalidateOnFocus: false,
        revalidateOnReconnect: false,
    })
}

function getPage(str: string) {
    const strArr = str.split(SPLIT)
    const provider = providerManager.get(strArr[0])
    if (!provider) throw new Error("Provider not found")
    const index = parseInt(strArr[1], 10);
    const search = strArr[2];
    const limit = parseInt(strArr[3], 10);
    return provider.getPage(index, limit, {filter: search})
}