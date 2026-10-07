use crate::error::{AppError, AppResult};
use crate::models::{AudioDeviceInfo, SystemAudioState};
use windows::core::{ComInterface, IUnknown, IUnknown_Vtbl, Interface, GUID, PCWSTR};
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eCommunications, eConsole, eMultimedia, eRender, IMMDevice, IMMDeviceEnumerator,
    MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
};
use windows::Win32::System::Com::StructuredStorage::{
    PropVariantClear, PropVariantToString, PROPVARIANT,
};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL, STGM_READ};

fn audio_error(context: &str, error: windows::core::Error) -> AppError {
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
    use windows::core::PWSTR;
    use windows::Win32::System::Com::{
        CoTaskMemAlloc,
        StructuredStorage::{PROPVARIANT_0, PROPVARIANT_0_0, PROPVARIANT_0_0_0},
    };
    use windows::Win32::System::Variant::VT_LPWSTR;

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
        unsafe extern "system" fn(*mut core::ffi::c_void, PCWSTR, i32) -> windows::core::HRESULT,
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
