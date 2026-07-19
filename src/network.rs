use std::collections::HashSet;
use std::sync::OnceLock;

use windows::Devices::WiFi::*;
use windows::Networking::Connectivity::*;
use windows::Security::Credentials::PasswordCredential;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::NetworkManagement::WiFi::*;
use windows_core::HSTRING;

fn async_runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

fn block_on_async<F: std::future::Future>(f: F) -> F::Output {
    async_runtime().block_on(f)
}

pub fn check_internet() -> bool {
    block_on_async(async {
        let profile = match NetworkInformation::GetInternetConnectionProfile() {
            Ok(p) => p,
            Err(_) => return false,
        };
        match profile.GetNetworkConnectivityLevel() {
            Ok(level) => level == NetworkConnectivityLevel::InternetAccess,
            Err(_) => false,
        }
    })
}

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub ssid: String,
    pub signal_bars: u8,
    pub is_secure: bool,
    pub is_connected: bool,
    pub has_saved_profile: bool,
}

async fn get_adapter() -> Option<WiFiAdapter> {
    let access = WiFiAdapter::RequestAccessAsync().ok()?.await.ok()?;
    if access != WiFiAccessStatus::Allowed {
        return None;
    }
    let adapters = WiFiAdapter::FindAllAdaptersAsync().ok()?.await.ok()?;
    adapters.GetAt(0).ok()
}

async fn get_connected_ssid(adapter: &WiFiAdapter) -> Option<String> {
    let adapter2 = adapter.NetworkAdapter().ok()?;
    let profile = adapter2.GetConnectedProfileAsync().ok()?.await.ok()?;
    Some(profile.ProfileName().ok()?.to_string())
}

fn is_network_secure(net: &WiFiAvailableNetwork) -> bool {
    let settings = match net.SecuritySettings() {
        Ok(s) => s,
        Err(_) => return false,
    };
    match settings.NetworkAuthenticationType() {
        Ok(auth) => {
            auth != NetworkAuthenticationType::None && auth != NetworkAuthenticationType::Unknown
        }
        Err(_) => false,
    }
}

async fn do_scan(saved_profiles: &HashSet<String>) -> Vec<NetworkInfo> {
    let adapter = match get_adapter().await {
        Some(a) => a,
        None => return Vec::new(),
    };

    let scan_op = match adapter.ScanAsync() {
        Ok(op) => op,
        Err(_) => return Vec::new(),
    };
    let _ = scan_op.await;

    let report = match adapter.NetworkReport() {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let networks = match report.AvailableNetworks() {
        Ok(n) => n,
        Err(_) => return Vec::new(),
    };
    let count = networks.Size().unwrap_or(0);

    let mut raw: Vec<(String, u8, bool)> = Vec::new();
    let mut seen = HashSet::new();

    for i in 0..count {
        let net = match networks.GetAt(i) {
            Ok(n) => n,
            Err(_) => continue,
        };

        let ssid = match net.Ssid() {
            Ok(s) => s.to_string(),
            Err(_) => continue,
        };
        if ssid.is_empty() || seen.contains(&ssid) {
            continue;
        }
        seen.insert(ssid.clone());

        let signal = net.SignalBars().unwrap_or(0);
        let is_secure = is_network_secure(&net);

        raw.push((ssid, signal, is_secure));
    }

    let connected_ssid = get_connected_ssid(&adapter).await;

    let mut results: Vec<NetworkInfo> = raw
        .into_iter()
        .map(|(ssid, signal_bars, is_secure)| {
            let is_connected = connected_ssid.as_deref() == Some(ssid.as_str());
            let has_saved_profile = saved_profiles.contains(&ssid);
            NetworkInfo {
                ssid,
                signal_bars,
                is_secure,
                is_connected,
                has_saved_profile,
            }
        })
        .collect();

    results.sort_by(|a, b| {
        b.is_connected
            .cmp(&a.is_connected)
            .then(b.signal_bars.cmp(&a.signal_bars))
    });

    results
}

async fn do_connect(ssid: &str, password: Option<&str>) -> bool {
    let adapter = match get_adapter().await {
        Some(a) => a,
        None => return false,
    };

    let scan_op = match adapter.ScanAsync() {
        Ok(op) => op,
        Err(_) => return false,
    };
    let _ = scan_op.await;

    let report = match adapter.NetworkReport() {
        Ok(r) => r,
        Err(_) => return false,
    };
    let networks = match report.AvailableNetworks() {
        Ok(n) => n,
        Err(_) => return false,
    };
    let count = networks.Size().unwrap_or(0);

    let target_idx = (0..count).find_map(|i| {
        let net = networks.GetAt(i).ok()?;
        let net_ssid = net.Ssid().ok()?.to_string();
        if net_ssid == ssid {
            Some(i)
        } else {
            None
        }
    });

    let target_idx = match target_idx {
        Some(i) => i,
        None => return false,
    };

    let target = match networks.GetAt(target_idx) {
        Ok(n) => n,
        Err(_) => return false,
    };

    let recon = WiFiReconnectionKind::Automatic;

    let result: WiFiConnectionResult = if let Some(pw) = password {
        let cred = PasswordCredential::new().unwrap();
        let _ = cred.SetPassword(&HSTRING::from(pw));
        match adapter.ConnectWithPasswordCredentialAsync(&target, recon, &cred) {
            Ok(op) => match op.await {
                Ok(r) => r,
                Err(_) => return false,
            },
            Err(_) => return false,
        }
    } else {
        match adapter.ConnectAsync(&target, recon) {
            Ok(op) => match op.await {
                Ok(r) => r,
                Err(_) => return false,
            },
            Err(_) => return false,
        }
    };

    result.ConnectionStatus().unwrap_or(WiFiConnectionStatus::UnspecifiedFailure)
        == WiFiConnectionStatus::Success
}

async fn do_disconnect() -> bool {
    let adapter = match get_adapter().await {
        Some(a) => a,
        None => return false,
    };
    let _ = adapter.Disconnect();
    true
}

fn get_saved_wifi_profiles() -> HashSet<String> {
    unsafe {
        let mut handle = HANDLE::default();
        let mut version = 0u32;
        if WlanOpenHandle(2, None, &mut version, &mut handle) != 0 {
            return HashSet::new();
        }

        let mut interface_list = std::ptr::null_mut();
        if WlanEnumInterfaces(handle, None, &mut interface_list) != 0 {
            let _ = WlanCloseHandle(handle, None);
            return HashSet::new();
        }

        let list = &*interface_list;
        if list.dwNumberOfItems == 0 {
            WlanFreeMemory(interface_list as *const _);
            let _ = WlanCloseHandle(handle, None);
            return HashSet::new();
        }

        let iface_guid = list.InterfaceInfo.as_ptr().read().InterfaceGuid;
        WlanFreeMemory(interface_list as *const _);

        let mut profile_list = std::ptr::null_mut();
        if WlanGetProfileList(handle, &iface_guid, None, &mut profile_list) != 0 {
            let _ = WlanCloseHandle(handle, None);
            return HashSet::new();
        }

        let profiles = &*profile_list;
        let count = profiles.dwNumberOfItems as usize;
        let mut result = HashSet::new();

        for i in 0..count {
            let name = profiles
                .ProfileInfo
                .as_ptr()
                .add(i)
                .read()
                .strProfileName;
            let len = name.iter().take_while(|&&c| c != 0).count();
            let s: String = name[..len].iter().map(|c| *c as u8 as char).collect();
            if !s.is_empty() {
                result.insert(s);
            }
        }

        WlanFreeMemory(profile_list as *const _);
        let _ = WlanCloseHandle(handle, None);
        result
    }
}

pub fn sync_scan() -> Vec<NetworkInfo> {
    let saved = get_saved_wifi_profiles();
    block_on_async(do_scan(&saved))
}

pub fn sync_connect(ssid: &str, password: Option<&str>) -> bool {
    let ssid_owned = ssid.to_owned();
    let pw_owned = password.map(|s| s.to_owned());
    block_on_async(async move {
        do_connect(&ssid_owned, pw_owned.as_deref()).await
    })
}

pub fn sync_disconnect() -> bool {
    block_on_async(do_disconnect())
}
