use just_enough_vcs::vcs::env::current_cfg_dir;

const IP_HISTORY_NAME: &str = "ip_history.txt";

pub struct IpAddressHistory {
    pub recent_ip_address: Vec<String>,
}

pub async fn get_recent_ip_address() -> Vec<String> {
    if let Some(local) = current_cfg_dir() {
        let path = local.join(IP_HISTORY_NAME);
        match tokio::fs::read_to_string(path).await {
            Ok(content) => content.lines().map(String::from).collect(),
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    }
}

pub async fn insert_recent_ip_address(ip: impl Into<String>) {
    let ip = ip.into();
    if let Some(local) = current_cfg_dir() {
        let path = local.join(IP_HISTORY_NAME);
        let mut recent_ips = get_recent_ip_address().await;
        recent_ips.retain(|existing_ip| existing_ip != &ip);
        recent_ips.insert(0, ip);
        if recent_ips.len() > 8 {
            recent_ips.truncate(8);
        }
        let content = recent_ips.join("\n");
        let _ = tokio::fs::write(path, content).await;
    }
}
