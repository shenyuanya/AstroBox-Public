export interface BTDeviceInfo {
    name: string,
    addr: string,
    connect_type: string
}

export interface MiWearState {
    name: string,
    addr: string,
    connect_type: string
    authkey?: string,
    codename?: string,
    disconnected?: boolean,
}

// 定义充电状态的枚举类型
export enum ChargeStatus {
    Unknown,
    Charging = "Charging",
    NotCharging = "NotCharging",
    Full = "Full",
}

// 定义系统状态的接口
export interface BatteryStatus {
    capacity: number;
    charge_status: ChargeStatus;
    charge_info?: ChargeInfo;
}

export interface ChargeInfo {
    state: number,
    timestamp?: number,
}

export interface SendMassCallBackData {
    progress: number,
    total_parts: number,
    current_part_num: number,
    actual_data_payload_len: number
}

// 定义应用信息的接口
export interface AppInfo {
    package_name: string;
    fingerprint: number[];
    version_code: number;
    can_remove: boolean;
    app_name: string;
}

// 定义系统信息的接口
export interface SystemInfo {
    serial_number: string;
    firmware_version: string;
    imei: string;
    model: string;
}

// 定义表盘信息的接口
export interface WatchfaceInfo {
    id: string;
    name: string;
    is_current: boolean;
    can_remove: boolean;
    version_code: number;
    can_edit: boolean;
    background_color: string;
    background_image: string;
    style: string;
    background_image_list: string[];
}
