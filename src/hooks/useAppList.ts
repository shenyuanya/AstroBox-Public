import { providerManager } from "@/community/manager";
import useSWRInfinite from 'swr/infinite';

const SPLIT = "__split__"
export default function useAppList(provider?: string, search?: string, filter?: string[], limit: number = 10) {
    return useSWRInfinite((index) => {
        if (!provider) return null
        if (!search) search = ""
        if (!filter) filter = []
        return provider + SPLIT + index + SPLIT + search + SPLIT + limit + SPLIT + filter?.toSorted()?.join("|你妈|");
    }, getPage, {
        dedupingInterval: 100000,
        revalidateOnFocus: false,
        revalidateOnReconnect: false,
        revalidateFirstPage: false,
    })
}

function getPage(str: string) {
    console.log(1)
    const strArr = str.split(SPLIT)
    const provider = providerManager.get(strArr[0])
    if (!provider) throw new Error("Provider not found")
    const index = parseInt(strArr[1], 10);
    const filter = strArr[2];
    const limit = parseInt(strArr[3], 10);
    const category = strArr[4].length > 0 ? strArr[4].split("|你妈|") : [];
    return provider.getPage(index, limit, { filter, category })
}