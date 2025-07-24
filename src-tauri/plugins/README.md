# AstroBox Custom Plugins
`btclassic-spp`: 适用于Android的经典蓝牙(SPP)接口绑定实现
## 疑难杂症 & 报错解决
### No matching configuration of project :tauri-android was found.
解决方案：新建`btclassic-spp/android/.tauri/tauri-api`文件夹，克隆[https://github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri)存储库，将里面的所有内容复制到刚刚新建的文件夹中，Rebuild即可