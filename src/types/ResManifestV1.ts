export interface Item {
    name?: string;
    restype?: string;
    description?: string;
    preview?: Array<string>;
    icon?: string;
    source_url?: string;
    author?: Array<Author>;
    _bandbbs_ext_supported_device?: string;
    _bandbbs_ext_resource_id?: number;
}

export interface Author {
    name?: string;
    author_url?: string;
}

export interface ResDownloadInfoV1 {
    version: string;
    file_name: string;
}

export interface ResourceManifestV1 {
    item: Item;
    downloads: { [key: string]: ResDownloadInfoV1 };
}