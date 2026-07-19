use std::sync::OnceLock;
use windows::core::HSTRING;
use windows::Foundation::IPropertyValue;
use windows::Storage::Streams::DataReader;
use windows::System::{KnownUserProperties, User, UserPictureSize};
use windows_core::Interface;

#[derive(Debug, Clone)]
pub struct ProfileInfo {
    pub display_name: String,
    pub principal_name: String,
    pub avatar: Option<(Vec<u8>, u32, u32)>,
}

fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

fn block_on_async<F: std::future::Future>(f: F) -> F::Output {
    runtime().block_on(f)
}

pub fn get_profile_info() -> ProfileInfo {
    match block_on_async(fetch_winrt_profile()) {
        Some(info) => info,
        None => fallback_profile(),
    }
}

fn fallback_profile() -> ProfileInfo {
    let display_name = get_local_display_name();
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "User".to_string());
    let domain = std::env::var("USERDOMAIN").unwrap_or_else(|_| "Local".to_string());
    let principal_name = format!("{}\\{}", domain, username);
    ProfileInfo {
        display_name,
        principal_name,
        avatar: None,
    }
}

fn get_local_display_name() -> String {
    unsafe {
        use windows::Win32::Security::Authentication::Identity::{GetUserNameExW, NameDisplay};

        let mut size = 0u32;
        GetUserNameExW(NameDisplay, None, &mut size);
        if size == 0 {
            return std::env::var("USERNAME").unwrap_or_else(|_| "User".to_string());
        }
        let mut buf = vec![0u16; size as usize];
        let success =
            GetUserNameExW(NameDisplay, Some(windows_core::PWSTR(buf.as_mut_ptr())), &mut size);
        if success && size > 0 {
            let name = String::from_utf16_lossy(&buf[..size as usize]);
            if !name.is_empty() {
                return name;
            }
        }
    }
    std::env::var("USERNAME").unwrap_or_else(|_| "User".to_string())
}

fn extract_string_property(
    props: &windows::Foundation::Collections::IPropertySet,
    key: &HSTRING,
) -> Option<String> {
    let val = props.Lookup(key).ok()?;
    let prop_val: IPropertyValue = val.cast().ok()?;
    let hstr: HSTRING = prop_val.GetString().ok()?;
    let s = hstr.to_string();
    if s.is_empty() { None } else { Some(s) }
}

async fn fetch_winrt_profile() -> Option<ProfileInfo> {
    let users = User::FindAllAsync().ok()?.await.ok()?;
    if users.Size().ok()? == 0 {
        return None;
    }
    let current_user = users.GetAt(0).ok()?;

    let display_name_prop = KnownUserProperties::DisplayName().ok()?;
    let account_name_prop = KnownUserProperties::AccountName().ok()?;
    let domain_name_prop = KnownUserProperties::DomainName().ok()?;

    let prop_names = windows_collections::IVectorView::<HSTRING>::from(vec![
        display_name_prop.clone(),
        account_name_prop.clone(),
        domain_name_prop.clone(),
    ]);

    let props = current_user.GetPropertiesAsync(&prop_names).ok()?.await.ok()?;

    let display_name = extract_string_property(&props, &display_name_prop);
    let account_name = extract_string_property(&props, &account_name_prop);
    let domain_name = extract_string_property(&props, &domain_name_prop);

    let display_name = display_name.unwrap_or_else(get_local_display_name);
    let principal_name = match (&account_name, &domain_name) {
        (Some(acct), Some(dom)) if !acct.is_empty() => acct.clone(),
        (Some(_), Some(dom)) if !dom.is_empty() => format!("{}\\{}", dom, display_name),
        (Some(acct), None) if !acct.is_empty() => acct.clone(),
        (None, Some(dom)) if !dom.is_empty() => format!("{}\\{}", dom, display_name),
        _ => "Local Account".to_string(),
    };

    let avatar = fetch_avatar(&current_user).await;

    Some(ProfileInfo {
        display_name,
        principal_name,
        avatar,
    })
}

async fn fetch_avatar(user: &User) -> Option<(Vec<u8>, u32, u32)> {
    let stream_ref = user
        .GetPictureAsync(UserPictureSize::Size1080x1080)
        .ok()?;
    let stream_ref = stream_ref.await.ok()?;
    let stream = stream_ref.OpenReadAsync().ok()?.await.ok()?;
    let size = stream.Size().ok()? as usize;
    if size == 0 {
        return None;
    }

    let input_stream = stream.GetInputStreamAt(0).ok()?;
    let reader = DataReader::CreateDataReader(&input_stream).ok()?;
    reader.LoadAsync(size as u32).ok()?.await.ok()?;

    let mut img_bytes = vec![0u8; size];
    reader.ReadBytes(&mut img_bytes).ok()?;

    let img = image::load_from_memory(&img_bytes).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Some((rgba.into_raw(), w, h))
}
