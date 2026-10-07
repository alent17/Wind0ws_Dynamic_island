//! Thread-bound Windows Runtime ownership. Declare before any COM interfaces.
use std::{marker::PhantomData, rc::Rc};
use windows::Win32::{
    Foundation::RPC_E_CHANGED_MODE,
    System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED, RO_INIT_SINGLETHREADED},
};

/// Neither Send nor Sync: initialization and teardown must use the same thread.
pub(crate) struct Apartment(PhantomData<Rc<()>>);

impl Apartment {
    pub(crate) fn enter() -> windows::core::Result<Self> {
        // Preserve an existing STA (e.g. Tauri's UI thread). A failed MTA
        // initialization owns no reference, so only the successful retry drops.
        unsafe {
            match RoInitialize(RO_INIT_MULTITHREADED) {
                Err(error) if error.code() == RPC_E_CHANGED_MODE => {
                    RoInitialize(RO_INIT_SINGLETHREADED)?;
                }
                result => result?,
            }
        }
        Ok(Self(PhantomData))
    }
}

impl Drop for Apartment {
    fn drop(&mut self) {
        // windows 0.52 maps both S_OK and S_FALSE to Ok; both need balancing.
        unsafe { RoUninitialize() };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Com::{
        CoGetApartmentType, CoInitializeEx, CoUninitialize, APTTYPE, APTTYPEQUALIFIER, APTTYPE_MTA,
        COINIT_APARTMENTTHREADED,
    };

    fn apartment_type() -> windows::core::Result<APTTYPE> {
        let mut kind = APTTYPE::default();
        let mut qualifier = APTTYPEQUALIFIER::default();
        unsafe { CoGetApartmentType(&mut kind, &mut qualifier)? };
        Ok(kind)
    }

    #[test]
    fn balances_nested_success_and_early_error() {
        std::thread::spawn(|| {
            assert!(apartment_type().is_err());
            {
                let _outer = Apartment::enter().unwrap();
                assert_eq!(apartment_type().unwrap(), APTTYPE_MTA);
                let fail = || -> windows::core::Result<()> {
                    let _inner = Apartment::enter()?;
                    Err(windows::core::Error::from(RPC_E_CHANGED_MODE))
                };
                assert!(fail().is_err());
                assert_eq!(apartment_type().unwrap(), APTTYPE_MTA);
            }
            assert!(apartment_type().is_err());
        })
        .join()
        .unwrap();
    }

    #[test]
    fn preserves_existing_sta_without_uninitializing_its_owner() {
        std::thread::spawn(|| unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap();
            let existing = apartment_type().unwrap();
            {
                let _guard = Apartment::enter().unwrap();
            }
            assert_eq!(apartment_type().unwrap(), existing);
            CoUninitialize();
            assert!(apartment_type().is_err());
        })
        .join()
        .unwrap();
    }
}
