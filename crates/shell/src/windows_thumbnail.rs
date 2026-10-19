//! COM implementation of `IThumbnailProvider` and `IInitializeWithStream`.
//!
//! Safety policy: nothing here may take down `explorer.exe` (or the `dllhost.exe` surrogate that
//! hosts thumbnail handlers). There is no `unwrap`, `expect` or `panic!`, every COM entry point
//! runs inside [`guard`], and every failure is reported as `E_FAIL`, so Explorer falls back to the
//! default icon.
#![allow(unsafe_code, non_snake_case, unsafe_op_in_unsafe_fn)]

use std::ffi::c_void;
use std::mem::{offset_of, zeroed};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::{addr_of_mut, null_mut};
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};

use windows_sys::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_FAIL, E_NOINTERFACE, E_OUTOFMEMORY, E_POINTER, E_UNEXPECTED, S_FALSE, S_OK,
};
use windows_sys::Win32::Graphics::Gdi::{DeleteObject, HBITMAP};
use windows_sys::Win32::Graphics::GdiPlus::{
    GdipCreateBitmapFromStream, GdipCreateHBITMAPFromBitmap, GdipDisposeImage, GdiplusShutdown, GdiplusStartup, GdiplusStartupInput,
};
use windows_sys::Win32::UI::Shell::SHCreateMemStream;
use windows_sys::core::{GUID, HRESULT};

use crate::psd;
use crate::registration::{self};

/// CLSID of the PhotoCraft PSD thumbnail provider (same value as `registration::CLSID_STR`).
const CLSID: GUID = GUID::from_u128(0x8D6B2D5C_3F41_4B7A_9C0E_5A17E2B94D63);

/// Never read more than this from the start of a file: its image resources have to fit.
const MAX_PREFIX_BYTES: usize = 16 * 1024 * 1024;
/// The first read, enough for the header of nearly every file and often the whole resources.
const FIRST_READ_BYTES: usize = 4096;
/// `HRESULT_FROM_WIN32(ERROR_ALREADY_INITIALIZED)`, as `IInitializeWithStream` specifies.
const E_ALREADY_INITIALIZED: HRESULT = 0x800700B7u32 as HRESULT;
/// `WTS_ALPHATYPE`: the bitmap has no meaningful alpha channel.
const WTSAT_RGB: i32 = 1;

const IID_IUNKNOWN: GUID = GUID::from_u128(0x00000000_0000_0000_C000_000000000046);
const IID_ICLASSFACTORY: GUID = GUID::from_u128(0x00000001_0000_0000_C000_000000000046);
const IID_IINITIALIZE_WITH_STREAM: GUID = GUID::from_u128(0xB824B49D_22AC_4161_AC8A_9916E8FA3F7F);
const IID_ITHUMBNAIL_PROVIDER: GUID = GUID::from_u128(0xE357FCCD_A995_4576_B01F_234630154E96);

/// Live objects plus server locks; backs `DllCanUnloadNow`.
static LIVE_COUNT: AtomicI32 = AtomicI32::new(0);

fn guid_eq(a: &GUID, b: &GUID) -> bool {
    a.data1 == b.data1 && a.data2 == b.data2 && a.data3 == b.data3 && a.data4 == b.data4
}

/// Runs `f`, turning a panic into `E_FAIL` instead of unwinding into COM callers.
fn guard(f: impl FnOnce() -> HRESULT) -> HRESULT {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or(E_FAIL)
}

// ---------------------------------------------------------------- vtables

#[repr(C)]
struct IUnknownVtbl {
    query_interface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
}

/// Only the `Read` slot of `IStream` (inherited from `ISequentialStream`) is needed.
#[repr(C)]
struct IStreamVtbl {
    base: IUnknownVtbl,
    read: unsafe extern "system" fn(*mut c_void, *mut c_void, u32, *mut u32) -> HRESULT,
}

#[repr(C)]
struct InitializeWithStreamVtbl {
    base: IUnknownVtbl,
    initialize: unsafe extern "system" fn(*mut c_void, *mut c_void, u32) -> HRESULT,
}

#[repr(C)]
struct ThumbnailProviderVtbl {
    base: IUnknownVtbl,
    get_thumbnail: unsafe extern "system" fn(*mut c_void, u32, *mut HBITMAP, *mut i32) -> HRESULT,
}

#[repr(C)]
struct ClassFactoryVtbl {
    base: IUnknownVtbl,
    create_instance: unsafe extern "system" fn(*mut c_void, *mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    lock_server: unsafe extern "system" fn(*mut c_void, i32) -> HRESULT,
}

static INIT_VTBL: InitializeWithStreamVtbl =
    InitializeWithStreamVtbl { base: IUnknownVtbl { query_interface: init_query_interface, add_ref: init_add_ref, release: init_release }, initialize };

static THUMB_VTBL: ThumbnailProviderVtbl =
    ThumbnailProviderVtbl { base: IUnknownVtbl { query_interface: thumb_query_interface, add_ref: thumb_add_ref, release: thumb_release }, get_thumbnail };

static FACTORY_VTBL: ClassFactoryVtbl = ClassFactoryVtbl {
    base: IUnknownVtbl { query_interface: factory_query_interface, add_ref: factory_add_ref, release: factory_release },
    create_instance: factory_create_instance,
    lock_server: factory_lock_server,
};

// --------------------------------------------------------------- provider

/// One COM object exposing two interfaces, each with its own vtable pointer.
#[repr(C)]
struct Provider {
    init_vtbl: *const InitializeWithStreamVtbl,
    thumb_vtbl: *const ThumbnailProviderVtbl,
    refs: AtomicU32,
    /// Start of the PSD file, filled by `Initialize`.
    data: Mutex<Option<Vec<u8>>>,
}

impl Provider {
    /// A new provider with a reference count of one.
    fn create() -> *mut Provider {
        LIVE_COUNT.fetch_add(1, Ordering::SeqCst);
        Box::into_raw(Box::new(Provider { init_vtbl: &INIT_VTBL, thumb_vtbl: &THUMB_VTBL, refs: AtomicU32::new(1), data: Mutex::new(None) }))
    }
}

/// The object behind an `IThumbnailProvider` interface pointer.
unsafe fn from_thumb(this: *mut c_void) -> *mut Provider {
    (this as *mut u8).sub(offset_of!(Provider, thumb_vtbl)) as *mut Provider
}

unsafe fn provider_query(p: *mut Provider, riid: *const GUID, ppv: *mut *mut c_void) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    *ppv = null_mut();
    if p.is_null() || riid.is_null() {
        return E_POINTER;
    }
    let riid = &*riid;
    if guid_eq(riid, &IID_IUNKNOWN) || guid_eq(riid, &IID_IINITIALIZE_WITH_STREAM) {
        *ppv = addr_of_mut!((*p).init_vtbl).cast();
    } else if guid_eq(riid, &IID_ITHUMBNAIL_PROVIDER) {
        *ppv = addr_of_mut!((*p).thumb_vtbl).cast();
    } else {
        return E_NOINTERFACE;
    }
    (*p).refs.fetch_add(1, Ordering::SeqCst);
    S_OK
}

unsafe fn provider_add_ref(p: *mut Provider) -> u32 {
    (*p).refs.fetch_add(1, Ordering::SeqCst) + 1
}

unsafe fn provider_release(p: *mut Provider) -> u32 {
    let remaining = (*p).refs.fetch_sub(1, Ordering::SeqCst) - 1;
    if remaining == 0 {
        drop(Box::from_raw(p));
        LIVE_COUNT.fetch_sub(1, Ordering::SeqCst);
    }
    remaining
}

unsafe extern "system" fn init_query_interface(this: *mut c_void, riid: *const GUID, ppv: *mut *mut c_void) -> HRESULT {
    guard(|| provider_query(this as *mut Provider, riid, ppv))
}
unsafe extern "system" fn init_add_ref(this: *mut c_void) -> u32 {
    provider_add_ref(this as *mut Provider)
}
unsafe extern "system" fn init_release(this: *mut c_void) -> u32 {
    provider_release(this as *mut Provider)
}
unsafe extern "system" fn thumb_query_interface(this: *mut c_void, riid: *const GUID, ppv: *mut *mut c_void) -> HRESULT {
    guard(|| provider_query(from_thumb(this), riid, ppv))
}
unsafe extern "system" fn thumb_add_ref(this: *mut c_void) -> u32 {
    provider_add_ref(from_thumb(this))
}
unsafe extern "system" fn thumb_release(this: *mut c_void) -> u32 {
    provider_release(from_thumb(this))
}

/// `IInitializeWithStream::Initialize`: buffers the start of the file.
unsafe extern "system" fn initialize(this: *mut c_void, stream: *mut c_void, _mode: u32) -> HRESULT {
    guard(|| {
        let provider = this as *mut Provider;
        if provider.is_null() || stream.is_null() {
            return E_POINTER;
        }
        let Some(bytes) = read_prefix(stream) else {
            return E_FAIL;
        };
        match (*provider).data.lock() {
            Ok(mut slot) if slot.is_none() => {
                *slot = Some(bytes);
                S_OK
            }
            Ok(_) => E_ALREADY_INITIALIZED,
            Err(_) => E_FAIL,
        }
    })
}

/// Reads the start of `stream` that holds the PSD's image resources: only as much as the file's own
/// header says they occupy (never more than [`MAX_PREFIX_BYTES`]), so a multi-gigabyte document is
/// not read through to show a 160 px preview. `None` when it is not a PSD/PSB.
unsafe fn read_prefix(stream: *mut c_void) -> Option<Vec<u8>> {
    let vtbl = *(stream as *mut *const IStreamVtbl);
    if vtbl.is_null() {
        return None;
    }
    let mut out = Vec::new();
    let mut chunk = vec![0u8; 64 * 1024];
    let mut want = FIRST_READ_BYTES;
    while out.len() < want {
        let step = (want - out.len()).min(chunk.len());
        let mut read = 0u32;
        let hr = ((*vtbl).read)(stream, chunk.as_mut_ptr().cast(), step as u32, &mut read);
        if hr < 0 {
            return None;
        }
        if read == 0 {
            break; // End of stream.
        }
        out.extend_from_slice(chunk.get(..(read as usize).min(step))?);
        if out.len() >= want {
            // Enough to see the header: now the exact amount the image resources need.
            let (psd::Extent::Known(end) | psd::Extent::NeedAtLeast(end)) = psd::resources_extent(&out)?;
            if end > MAX_PREFIX_BYTES {
                return None;
            }
            want = end;
        }
    }
    Some(out)
}

/// `IThumbnailProvider::GetThumbnail`: the embedded JFIF as an `HBITMAP` that Explorer owns.
unsafe extern "system" fn get_thumbnail(this: *mut c_void, _cx: u32, phbmp: *mut HBITMAP, pdw_alpha: *mut i32) -> HRESULT {
    guard(|| {
        if this.is_null() || phbmp.is_null() || pdw_alpha.is_null() {
            return E_POINTER;
        }
        *phbmp = null_mut();
        let provider = &*from_thumb(this);
        let Ok(data) = provider.data.lock() else {
            return E_FAIL;
        };
        let Some(data) = data.as_deref() else {
            return E_UNEXPECTED; // `Initialize` was not called.
        };
        match psd::find_thumbnail_jpeg(data).and_then(|jpeg| jpeg_to_hbitmap(jpeg)) {
            Some(hbmp) => {
                *phbmp = hbmp;
                *pdw_alpha = WTSAT_RGB;
                S_OK
            }
            None => E_FAIL,
        }
    })
}

/// Decodes a JPEG with GDI+, which ships with Windows. Explorer scales the bitmap to the size it
/// wants.
unsafe fn jpeg_to_hbitmap(jpeg: &[u8]) -> Option<HBITMAP> {
    let stream = SHCreateMemStream(jpeg.as_ptr(), u32::try_from(jpeg.len()).ok()?);
    if stream.is_null() {
        return None;
    }

    let mut input: GdiplusStartupInput = zeroed();
    input.GdiplusVersion = 1;
    let mut token = 0usize;
    if GdiplusStartup(&mut token, &input, null_mut()) != 0 {
        release_stream(stream);
        return None;
    }

    let mut bitmap = null_mut();
    let mut hbmp: HBITMAP = null_mut();
    let mut ok = GdipCreateBitmapFromStream(stream, &mut bitmap) == 0 && !bitmap.is_null();
    if ok {
        // Opaque white background; a JPEG has no alpha anyway.
        ok = GdipCreateHBITMAPFromBitmap(bitmap, &mut hbmp, 0xFFFF_FFFF) == 0 && !hbmp.is_null();
    }
    if !bitmap.is_null() {
        GdipDisposeImage(bitmap.cast());
    }
    GdiplusShutdown(token);
    release_stream(stream);
    if ok {
        return Some(hbmp);
    }
    if !hbmp.is_null() {
        DeleteObject(hbmp);
    }
    None
}

unsafe fn release_stream(stream: *mut c_void) {
    let vtbl = *(stream as *mut *const IUnknownVtbl);
    if !vtbl.is_null() {
        ((*vtbl).release)(stream);
    }
}

// ---------------------------------------------------------- class factory

#[repr(C)]
struct Factory {
    vtbl: *const ClassFactoryVtbl,
    refs: AtomicU32,
}

unsafe extern "system" fn factory_query_interface(this: *mut c_void, riid: *const GUID, ppv: *mut *mut c_void) -> HRESULT {
    guard(|| {
        if ppv.is_null() || riid.is_null() {
            return E_POINTER;
        }
        *ppv = null_mut();
        if guid_eq(&*riid, &IID_IUNKNOWN) || guid_eq(&*riid, &IID_ICLASSFACTORY) {
            factory_add_ref(this);
            *ppv = this;
            S_OK
        } else {
            E_NOINTERFACE
        }
    })
}

unsafe extern "system" fn factory_add_ref(this: *mut c_void) -> u32 {
    (*(this as *mut Factory)).refs.fetch_add(1, Ordering::SeqCst) + 1
}

unsafe extern "system" fn factory_release(this: *mut c_void) -> u32 {
    let factory = this as *mut Factory;
    let remaining = (*factory).refs.fetch_sub(1, Ordering::SeqCst) - 1;
    if remaining == 0 {
        drop(Box::from_raw(factory));
        LIVE_COUNT.fetch_sub(1, Ordering::SeqCst);
    }
    remaining
}

unsafe extern "system" fn factory_create_instance(_this: *mut c_void, outer: *mut c_void, riid: *const GUID, ppv: *mut *mut c_void) -> HRESULT {
    guard(|| {
        if ppv.is_null() {
            return E_POINTER;
        }
        *ppv = null_mut();
        if !outer.is_null() {
            return CLASS_E_NOAGGREGATION;
        }
        let provider = Provider::create();
        if provider.is_null() {
            return E_OUTOFMEMORY;
        }
        let hr = provider_query(provider, riid, ppv);
        provider_release(provider); // Drops the creation reference.
        hr
    })
}

unsafe extern "system" fn factory_lock_server(_this: *mut c_void, lock: i32) -> HRESULT {
    LIVE_COUNT.fetch_add(if lock != 0 { 1 } else { -1 }, Ordering::SeqCst);
    S_OK
}

// ------------------------------------------------------------ DLL exports

/// COM entry point: hands out the class factory for [`CLSID`].
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const GUID, riid: *const GUID, ppv: *mut *mut c_void) -> HRESULT {
    guard(|| {
        if rclsid.is_null() || riid.is_null() || ppv.is_null() {
            return E_POINTER;
        }
        *ppv = null_mut();
        if !guid_eq(&*rclsid, &CLSID) {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        LIVE_COUNT.fetch_add(1, Ordering::SeqCst);
        let factory = Box::into_raw(Box::new(Factory { vtbl: &FACTORY_VTBL, refs: AtomicU32::new(1) }));
        let hr = factory_query_interface(factory.cast(), riid, ppv);
        factory_release(factory.cast());
        hr
    })
}

/// COM entry point: whether Windows may unload the DLL.
#[unsafe(no_mangle)]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    if LIVE_COUNT.load(Ordering::SeqCst) <= 0 { S_OK } else { S_FALSE }
}

/// `regsvr32` entry point: registers the provider for the current user.
#[unsafe(no_mangle)]
pub extern "system" fn DllRegisterServer() -> HRESULT {
    guard(|| match registration::current_module_path() {
        Some(path) if registration::register(&path, true).is_ok() => S_OK,
        _ => E_FAIL,
    })
}

/// `regsvr32 /u` entry point: removes the registration.
#[unsafe(no_mangle)]
pub extern "system" fn DllUnregisterServer() -> HRESULT {
    guard(|| match registration::unregister() {
        Ok(()) => S_OK,
        Err(_) => E_FAIL,
    })
}
