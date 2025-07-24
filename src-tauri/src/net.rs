use once_cell::sync::Lazy;
use reqwest::{Client, ClientBuilder, NoProxy, Proxy};
use sysproxy::Sysproxy;

pub static DEFAULT_CLIENT: Lazy<Client> = Lazy::new(build_client);

fn build_client() -> Client {
    default_client_builder().build().unwrap()
}

pub fn default_client() -> Client {
    DEFAULT_CLIENT.clone()
}

pub fn default_client_builder() -> ClientBuilder {
    #[cfg(not(target_os = "android"))]
    if let Ok(proxy)= Sysproxy::get_system_proxy(){
        if proxy.enable{
            return Client::builder().proxy(Proxy::all(format!("{}:{}",proxy.host,proxy.port)).unwrap().no_proxy(NoProxy::from_string(&proxy.bypass.as_str())));
        }
    }
    Client::builder()
}
