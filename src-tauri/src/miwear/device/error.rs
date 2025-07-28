use crate::pb::protocol::PrepareStatus;

pub fn get_prepare_error_info(code: i32) -> String {
    match PrepareStatus::try_from(code) {
        Ok(PrepareStatus::Busy) => "设备繁忙，请尝试重启设备，然后再试".to_string(),
        Ok(PrepareStatus::Downgrade) => "出现Downgrade错误，可能是由于表盘id重复造成的，建议修改表盘id并检查设备是否正常连接后再试".to_string(),
        Ok(PrepareStatus::Duplicated) => "出现Duplicated错误，你可能在尝试重复安装已安装的资源".to_string(),
        Ok(PrepareStatus::ExceedQuantityLimit) => "出现ExceedQuantityLimit错误，可能是安装的资源数量超出了设备限制，请删除一些资源后再试".to_string(),
        Ok(PrepareStatus::Failed) => "出现未知错误".to_string(),
        Ok(PrepareStatus::LowBattery) => "电量过低导致安装失败，充个电吧哥们".to_string(),
        Ok(PrepareStatus::LowStorage) => "存储空间不足导致安装失败，请检查安装的表盘数量或快应用数量有没有超出系统最大限制，并确保可用空间充足".to_string(),
        Ok(PrepareStatus::NetworkError) => "网络错误导致安装失败，如果设备支持eSIM，建议开启移动数据再试".to_string(),
        Ok(PrepareStatus::OpNotSupport) => "设备暂不支持该指令".to_string(),
        _ => "未知错误".to_string(),
    }
}