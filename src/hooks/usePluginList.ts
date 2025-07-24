import useSWRInfinite from 'swr/infinite'
import { providerManager } from '@/pluginstore/manager'

const SPLIT = '__split__'

export default function usePluginList(provider?: string, search?: string, limit = 20) {
    return useSWRInfinite((index) => {
        if (!provider) return null
        return `${provider}${SPLIT}${index}${SPLIT}${search ?? ''}${SPLIT}${limit}`
    }, getPage, {
        revalidateFirstPage: false,
        revalidateOnFocus: false,
        revalidateOnReconnect: false,
    })
}

async function getPage(key: string) {
    const [providerName, indexStr, search, limitStr] = key.split(SPLIT)
    const provider = providerManager.get(providerName)
    if (!provider) throw new Error('Provider not found')
    const index = parseInt(indexStr, 10)
    const limit = parseInt(limitStr, 10)
    return provider.getPage(index, limit, search)
}
