import {StaticImport} from "next/dist/shared/lib/get-img-props";

export interface BannerItem {
    background: string | StaticImport;
    title: string;
    description: string;
    foreground: string | StaticImport;
    button: {
        text?: string;
        url: string;
    }
}