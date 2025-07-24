import { listen } from "@tauri-apps/api/event";

export function initOpenResourceListener(router: any) {
    listen("open-resource", (data) => {
        var payload = data.payload as any;

        router.push({
            pathname: '/community/product-info/product-info',
            query: { name: payload.resname, provider: payload.provider_name },
        });
    })
}