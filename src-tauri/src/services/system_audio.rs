use crate::error::{AppError, AppResult};
use crate::models::{AppAudioSession, AudioDeviceInfo, SystemAudioState};
use windows052::core::{ComInterface, IUnknown, IUnknown_Vtbl, Interface, GUID, PCWSTR};
use windows052::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows052::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows052::Win32::Media::Audio::{
    eCommunications, eConsole, eMultimedia, eRender, IMMDevice, IMMDeviceEnumerator,
    MMDeviceEnumerator, DEVICE_STATE_ACTIVE, IAudioSessionControl2, IAudioSessionManager2,
    ISimpleAudioVolume, AudioSessionStateExpired,
};
use windows052::Win32::System::Com::StructuredStorage::{
    PropVariantClear, PropVariantToString, PROPVARIANT,
};
use windows052::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL, STGM_READ};

fn application_icon(path: &std::path::Path) -> Option<String> {
    use base64::Engine;
    use std::os::windows::ffi::OsStrExt;
    use std::sync::{Mutex, OnceLock};
    use windows052::Win32::Foundation::HANDLE;
    use windows052::Win32::Graphics::Gdi::{CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HDC, HGDIOBJ};
    use windows052::Win32::UI::Shell::ExtractIconExW;
    use windows052::Win32::UI::WindowsAndMessaging::{DestroyIcon, DrawIconEx, HICON, DI_NORMAL};

    static CACHE: OnceLock<Mutex<std::collections::HashMap<std::path::PathBuf, Option<String>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    if let Ok(cache) = cache.lock() { if let Some(icon) = cache.get(path) { return icon.clone(); } }
    let result = unsafe {
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut icon = HICON::default();
        if ExtractIconExW(PCWSTR(wide.as_ptr()), 0, Some(&mut icon), None, 1) == 0 || icon.0 == 0 { None } else {
            let dc = CreateCompatibleDC(HDC::default());
            let mut info = BITMAPINFO::default();
            info.bmiHeader = BITMAPINFOHEADER { biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32, biWidth: 32, biHeight: -32, biPlanes: 1, biBitCount: 32, biCompression: BI_RGB.0, ..Default::default() };
            let mut pixels = std::ptr::null_mut();
            let bitmap = if dc.0 != 0 { CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut pixels, HANDLE::default(), 0).ok() } else { None };
            let png = if let Some(bitmap) = bitmap {
                let old = SelectObject(dc, HGDIOBJ(bitmap.0));
                std::ptr::write_bytes(pixels, 0, 32 * 32 * 4);
                let drawn = DrawIconEx(dc, 0, 0, icon, 32, 32, 0, None, DI_NORMAL).is_ok();
                let mut rgba = std::slice::from_raw_parts(pixels.cast::<u8>(), 32 * 32 * 4).to_vec();
                SelectObject(dc, old);
                DeleteObject(HGDIOBJ(bitmap.0));
                if drawn && rgba.chunks_exact(4).any(|pixel| pixel[3] != 0) {
                    for pixel in rgba.chunks_exact_mut(4) {
                        pixel.swap(0, 2);
                        let alpha = u32::from(pixel[3]);
                        if alpha > 0 && alpha < 255 { for channel in &mut pixel[..3] { *channel = (u32::from(*channel) * 255 / alpha).min(255) as u8; } }
                    }
                    image::RgbaImage::from_raw(32, 32, rgba).and_then(|image| {
                        let mut output = std::io::Cursor::new(Vec::new());
                        image::DynamicImage::ImageRgba8(image).write_to(&mut output, image::ImageFormat::Png).ok()?;
                        Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(output.into_inner())))
                    })
                } else { None }
            } else { None };
            if dc.0 != 0 { DeleteDC(dc); }
            let _ = DestroyIcon(icon);
            png
        }
    };
    if let Ok(mut cache) = cache.lock() { if cache.len() >= 128 { cache.clear(); } cache.insert(path.to_owned(), result.clone()); }
    result
}

fn audio_error(context: &str, error: windows052::core::Error) -> AppError {
    AppError::business(5101, format!("{}: {}", context, error))
}

// Move-only ownership; windows 0.52's raw PROPVARIANT has no Drop implementation.
struct OwnedPropVariant(PROPVARIANT);

impl Drop for OwnedPropVariant {
    fn drop(&mut self) {
        if let Err(error) = unsafe { PropVariantClear(&mut self.0) } {
            tracing::warn!("Failed to clear audio device property: {error}");
        }
    }
}

fn audio_apartment() -> AppResult<super::apartment::Apartment> {
    super::apartment::Apartment::enter().map_err(|error| audio_error("无法初始化音频线程", error))
}

unsafe fn enumerator() -> AppResult<IMMDeviceEnumerator> {
    CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
        .map_err(|error| audio_error("无法创建音频设备枚举器", error))
}

unsafe fn device_id(device: &IMMDevice) -> AppResult<String> {
    let value = device
        .GetId()
        .map_err(|error| audio_error("无法读取音频设备 ID", error))?;
    let result = value
        .to_string()
        .map_err(|error| AppError::business(5101, format!("音频设备 ID 编码无效: {}", error)));
    CoTaskMemFree(Some(value.0.cast()));
    result
}

unsafe fn device_name(device: &IMMDevice) -> String {
    let Ok(store) = device.OpenPropertyStore(STGM_READ) else {
        return "音频输出设备".to_string();
    };
    let Ok(value) = store.GetValue(&PKEY_Device_FriendlyName) else {
        return "音频输出设备".to_string();
    };
    let value = OwnedPropVariant(value);
    property_name(value)
}

fn property_name(value: OwnedPropVariant) -> String {
    let mut buffer = [0u16; 512];
    if unsafe { PropVariantToString(&value.0, &mut buffer) }.is_err() {
        return "音频输出设备".to_string();
    }
    let length = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..length])
}

#[cfg(test)]
mod resource_tests {
    use super::*;
    use std::mem::ManuallyDrop;
    use windows052::core::PWSTR;
    use windows052::Win32::System::Com::{
        CoTaskMemAlloc,
        StructuredStorage::{PROPVARIANT_0, PROPVARIANT_0_0, PROPVARIANT_0_0_0},
    };
    use windows052::Win32::System::Variant::VT_LPWSTR;

    fn string_property(text: &str) -> OwnedPropVariant {
        let wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        let allocation = unsafe { CoTaskMemAlloc(wide.len() * size_of::<u16>()) }.cast::<u16>();
        assert!(!allocation.is_null());
        unsafe { std::ptr::copy_nonoverlapping(wide.as_ptr(), allocation, wide.len()) };
        OwnedPropVariant(PROPVARIANT {
            Anonymous: PROPVARIANT_0 {
                Anonymous: ManuallyDrop::new(PROPVARIANT_0_0 {
                    vt: VT_LPWSTR,
                    Anonymous: PROPVARIANT_0_0_0 {
                        pwszVal: PWSTR(allocation),
                    },
                    ..Default::default()
                }),
            },
        })
    }

    #[test]
    fn formats_owned_string_and_handles_insufficient_conversion_buffer() {
        assert_eq!(property_name(string_property("扬声器")), "扬声器");
        assert_eq!(
            property_name(string_property(&"x".repeat(600))),
            "音频输出设备"
        );
    }

    #[test]
    #[ignore = "Requires a Windows audio service; run explicitly for native device stress"]
    fn queries_device_properties_1000_times() {
        for _ in 0..1000 {
            let devices = list_audio_output_devices().unwrap();
            assert!(
                !devices.is_empty(),
                "No audio output device for this diagnostic"
            );
            assert!(devices
                .iter()
                .all(|device| !device.id.is_empty() && !device.name.is_empty()));
        }
    }

    #[test]
    #[ignore = "Requires the Windows audio service and an output device"]
    fn reads_application_sessions_from_the_default_device() {
        let sessions = list_app_audio_sessions().expect("application audio sessions");
        assert!(sessions.iter().all(|session| !session.device_id.is_empty() && session.volume_percent <= 100));
    }

    #[test]
    #[ignore = "Requires the Windows Notepad executable"]
    fn extracts_a_windows_program_icon_as_png() {
        use base64::Engine;
        let path = std::path::PathBuf::from(std::env::var_os("WINDIR").unwrap()).join("System32").join("notepad.exe");
        let icon = application_icon(&path).expect("Notepad should have an extractable icon");
        let bytes = base64::engine::general_purpose::STANDARD.decode(icon.strip_prefix("data:image/png;base64,").unwrap()).unwrap();
        let image = image::load_from_memory(&bytes).expect("icon should be a valid PNG");
        assert_eq!((image.width(), image.height()), (32, 32));
    }
}

pub fn get_system_audio_state() -> AppResult<SystemAudioState> {
    let _apartment = audio_apartment()?;
    unsafe {
        let enumerator = enumerator()?;
        let device = enumerator
            .GetDefaultAudioEndpoint(eRender, eMultimedia)
            .map_err(|error| audio_error("未找到默认音频输出设备", error))?;
        let endpoint: IAudioEndpointVolume = device
            .Activate(CLSCTX_ALL, None)
            .map_err(|error| audio_error("无法读取系统音量", error))?;
        let volume = endpoint
            .GetMasterVolumeLevelScalar()
            .map_err(|error| audio_error("无法读取系统音量", error))?;
        let muted = endpoint
            .GetMute()
            .map_err(|error| audio_error("无法读取静音状态", error))?
            .as_bool();
        Ok(SystemAudioState {
            volume_percent: (volume.clamp(0.0, 1.0) * 100.0).round() as u8,
            muted,
            device_id: device_id(&device)?,
            device_name: device_name(&device),
        })
    }
}

pub fn list_audio_output_devices() -> AppResult<Vec<AudioDeviceInfo>> {
    let _apartment = audio_apartment()?;
    unsafe {
        let enumerator = enumerator()?;
        let default = enumerator
            .GetDefaultAudioEndpoint(eRender, eMultimedia)
            .ok();
        let default_id = default.as_ref().and_then(|device| device_id(device).ok());
        let collection = enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .map_err(|error| audio_error("无法枚举音频输出设备", error))?;
        let mut devices = Vec::new();
        for index in 0..collection.GetCount().unwrap_or(0) {
            let Ok(device) = collection.Item(index) else {
                continue;
            };
            let Ok(id) = device_id(&device) else { continue };
            devices.push(AudioDeviceInfo {
                is_default: default_id.as_deref() == Some(id.as_str()),
                name: device_name(&device),
                id,
            });
        }
        devices.sort_by_key(|device| (!device.is_default, device.name.to_lowercase()));
        Ok(devices)
    }
}

pub fn set_system_volume(volume_percent: u8) -> AppResult<()> {
    let _apartment = audio_apartment()?;
    unsafe {
        let device = enumerator()?
            .GetDefaultAudioEndpoint(eRender, eMultimedia)
            .map_err(|error| audio_error("未找到默认音频输出设备", error))?;
        let endpoint: IAudioEndpointVolume = device
            .Activate(CLSCTX_ALL, None)
            .map_err(|error| audio_error("无法控制系统音量", error))?;
        endpoint
            .SetMasterVolumeLevelScalar(volume_percent.min(100) as f32 / 100.0, std::ptr::null())
            .map_err(|error| audio_error("设置系统音量失败", error))?;
        if volume_percent > 0 {
            endpoint
                .SetMute(false, std::ptr::null())
                .map_err(|error| audio_error("取消静音失败", error))?;
        }
        Ok(())
    }
}

unsafe fn audio_sessions() -> AppResult<(String, windows052::Win32::Media::Audio::IAudioSessionEnumerator)> {
    let device = enumerator()?.GetDefaultAudioEndpoint(eRender, eMultimedia)
        .map_err(|error| audio_error("未找到默认音频输出设备", error))?;
    let id = device_id(&device)?;
    let manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)
        .map_err(|error| audio_error("无法读取应用音量", error))?;
    let sessions = manager.GetSessionEnumerator().map_err(|error| audio_error("无法枚举应用音量", error))?;
    Ok((id, sessions))
}

pub fn list_app_audio_sessions() -> AppResult<Vec<AppAudioSession>> {
    let _apartment = audio_apartment()?;
    unsafe {
        let (device_id, sessions) = audio_sessions()?;
        let mut processes = sysinfo::System::new();
        let mut result: Vec<AppAudioSession> = Vec::new();
        for index in 0..sessions.GetCount().unwrap_or(0) {
            let Ok(control) = sessions.GetSession(index) else { continue };
            if control.GetState().ok() == Some(AudioSessionStateExpired) { continue; }
            let Ok(details) = control.cast::<IAudioSessionControl2>() else { continue };
            let Ok(volume) = control.cast::<ISimpleAudioVolume>() else { continue };
            let Ok(process_id) = details.GetProcessId() else { continue };
            let system_sounds = details.IsSystemSoundsSession().0 == 0;
            if result.iter().any(|item| item.process_id == process_id && item.system_sounds == system_sounds) { continue; }
            let name = if system_sounds { String::new() } else {
                let pid = sysinfo::Pid::from_u32(process_id);
                processes.refresh_process(pid);
                processes.process(pid)
                    .map(|process| process.name().trim_end_matches(".exe").to_string())
                    .filter(|name| !name.is_empty()).unwrap_or_else(|| format!("PID {process_id}"))
            };
            let Ok(level) = volume.GetMasterVolume() else { continue };
            let icon = if system_sounds { None } else { processes.process(sysinfo::Pid::from_u32(process_id)).and_then(|process| process.exe()).and_then(application_icon) };
            result.push(AppAudioSession { process_id, device_id: device_id.clone(), name, icon,
                volume_percent: (level.clamp(0.0, 1.0) * 100.0).round() as u8,
                muted: volume.GetMute().map(|value| value.as_bool()).unwrap_or(false), system_sounds });
        }
        result.sort_by(|a,b| a.system_sounds.cmp(&b.system_sounds).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        Ok(result)
    }
}

pub fn set_app_audio_volume(device_id_expected: &str, process_id: u32, system_sounds: bool, volume_percent: Option<u8>, muted: bool) -> AppResult<()> {
    let _apartment = audio_apartment()?;
    unsafe {
        let (device_id, sessions) = audio_sessions()?;
        if device_id != device_id_expected { return Err(AppError::business(5101, "音频设备已更改，请刷新应用列表")); }
        let mut found = false;
        for index in 0..sessions.GetCount().unwrap_or(0) {
            let Ok(control) = sessions.GetSession(index) else { continue };
            if control.GetState().ok() == Some(AudioSessionStateExpired) { continue; }
            let Ok(details) = control.cast::<IAudioSessionControl2>() else { continue };
            if details.GetProcessId().ok() != Some(process_id) || (details.IsSystemSoundsSession().0 == 0) != system_sounds { continue; }
            let Ok(volume) = control.cast::<ISimpleAudioVolume>() else { continue };
            if let Some(volume_percent) = volume_percent {
                volume.SetMasterVolume(volume_percent.min(100) as f32 / 100.0, std::ptr::null())
                    .map_err(|error| audio_error("设置应用音量失败", error))?;
            }
            volume.SetMute(muted, std::ptr::null()).map_err(|error| audio_error("设置应用静音状态失败", error))?;
            found = true;
        }
        if !found { return Err(AppError::business(5101, "应用音频会话已结束")); }
        Ok(())
    }
}

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq)]
struct IPolicyConfig(IUnknown);

unsafe impl Interface for IPolicyConfig {
    type Vtable = IPolicyConfig_Vtbl;
}

unsafe impl ComInterface for IPolicyConfig {
    const IID: GUID = GUID::from_u128(0xf8679f50_850a_41cf_9c72_430f290290c8);
}

#[repr(C)]
#[allow(non_snake_case)]
struct IPolicyConfig_Vtbl {
    base__: IUnknown_Vtbl,
    GetMixFormat: usize,
    GetDeviceFormat: usize,
    ResetDeviceFormat: usize,
    SetDeviceFormat: usize,
    GetProcessingPeriod: usize,
    SetProcessingPeriod: usize,
    GetShareMode: usize,
    SetShareMode: usize,
    GetPropertyValue: usize,
    SetPropertyValue: usize,
    SetDefaultEndpoint:
        unsafe extern "system" fn(*mut core::ffi::c_void, PCWSTR, i32) -> windows052::core::HRESULT,
    SetEndpointVisibility: usize,
}

pub fn set_default_audio_output(id: &str) -> AppResult<()> {
    let _apartment = audio_apartment()?;
    let wide: Vec<u16> = id.encode_utf16().chain(Some(0)).collect();
    unsafe {
        const POLICY_CONFIG_CLIENT: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);
        let policy: IPolicyConfig = CoCreateInstance(&POLICY_CONFIG_CLIENT, None, CLSCTX_ALL)
            .map_err(|error| audio_error("Windows 不支持切换默认音频设备", error))?;
        let set_default = Interface::vtable(&policy).SetDefaultEndpoint;
        for role in [eConsole, eMultimedia, eCommunications] {
            set_default(Interface::as_raw(&policy), PCWSTR(wide.as_ptr()), role.0)
                .ok()
                .map_err(|error| audio_error("切换默认音频设备失败", error))?;
        }
    }
    Ok(())
}
