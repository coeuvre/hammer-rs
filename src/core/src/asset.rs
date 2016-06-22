use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::path::Path;
use std::ops::Deref;
use std::ffi::CString;
use std::ptr;
use std::sync::Arc;
use std::sync::{Mutex, RwLock, RwLockReadGuard};

use Error;

use typemap::{TypeMap, Key};

use util::stb_image::*;
use util::cstr_to_string;

pub type AssetId = String;

pub trait AsAssetId {
    fn as_asset_id(&self) -> AssetId;
}

impl<'a> AsAssetId for &'a str {
    fn as_asset_id(&self) -> AssetId {
        self.to_string()
    }
}

pub trait Resource: Any + Send {}

pub trait Loadable: Sized {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
}

pub enum AssetState<A> {
    Unloaded,
    Loading,
    Loaded(Box<A>),
    LoadError(Error),
}

pub struct Asset<A> {
    id: AssetId,
    state: Arc<RwLock<AssetState<A>>>,
}

impl<A> Asset<A> {
    fn new(id: AssetId) -> Asset<A> {
        Asset {
            id: id,
            state: Arc::new(RwLock::new(AssetState::Unloaded)),
        }
    }

    pub fn id(&self) -> &AssetId {
        &self.id
    }

    pub fn borrow(&self) -> Option<AssetRef<A>> {
        // Panic if can't read
        let guard = self.state.try_read().unwrap();
        match *guard {
            AssetState::Loaded(_) => {
                Some(AssetRef {
                     guard: guard,
                  })
            }

            _ => None,
        }
    }

    pub fn load_with<F: FnOnce() -> Result<A, Error>>(&self, f: F) -> Result<(), Error>{
        let load = {
            let mut state = self.state.write().unwrap();
            match *state {
                AssetState::Loading => {
                    false
                }

                _ => {
                    *state = AssetState::Loading;
                    true
                }
            }
        };

        if load {
            let mut err = None;

            let new_state = match f() {
                Ok(asset) => AssetState::Loaded(Box::new(asset)),
                Err(e) => {
                    err = Some(format!("Failed to load asset: {}", e));
                    AssetState::LoadError(e)
                }
            };

            let mut state = self.state.write().unwrap();
            *state = new_state;

            if let Some(e) = err {
                Err(e.into())
            } else {
                Ok(())
            }
        } else {
            Err("Asset is loading".into())
        }
    }

    pub fn insert(&self, asset: A) -> Result<(), Error> {
        self.load_with(|| Ok(asset))
    }
}

impl<A: Loadable> Asset<A> {
    pub fn load<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        self.load_with(|| A::load(path))
    }
}

impl<A> Clone for Asset<A> {
    fn clone(&self) -> Self {
        Asset {
            id: self.id.clone(),
            state: self.state.clone(),
        }
    }
}

pub struct AssetRef<'a, A: 'a> {
    guard: RwLockReadGuard<'a, AssetState<A>>,
}

impl<'a, A: 'a> Deref for AssetRef<'a, A> {
    type Target = A;
    fn deref(&self) -> &A {
        match *self.guard {
            AssetState::Loaded(ref asset) => {
                asset
            }

            _ => unreachable!(),
        }
    }
}

pub fn asset<A: Resource>() -> AssetWrapper<A> {
    AssetWrapper {
        phantom: PhantomData,
    }
}

pub struct AssetWrapper<A> {
    phantom: PhantomData<A>,
}

impl<A: Resource> AssetWrapper<A> {
    pub fn get<I: AsAssetId>(&self, id: I) -> Asset<A> {
        let id = id.as_asset_id();
        ASSETS.slots.acquire::<A>(&id)
    }
}

#[derive(Clone)]
pub struct Texture {
    w: i32,
    h: i32,
    data: *mut u8,
}

impl Resource for Texture {}
unsafe impl Send for Texture {}

impl Loadable for Texture {
    fn load<P: AsRef<Path>>(path: P) -> Result<Texture, Error> {
        unsafe {
            let cstr = CString::new(&*path.as_ref().as_os_str().to_string_lossy()).unwrap();
            let mut w = 0;
            let mut h = 0;
            let data = stbi_load(cstr.as_ptr(), &mut w, &mut h, ptr::null_mut(), 4);
            if data != ptr::null_mut() {
                info!("Loaded texture `{}`.", path.as_ref().display());

                Ok(Texture {
                    w: w,
                    h: h,
                    data: data,
                })
            } else {
                Err(format!("Failed to load texture {}: {}", path.as_ref().display(), cstr_to_string(stbi_failure_reason())).into())
            }
        }
    }
}

impl Texture {
    pub fn size(&self) -> (i32, i32) {
        (self.w, self.h)
    }

    pub unsafe fn data(&self) -> *mut u8 {
        self.data
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            stbi_image_free(self.data);
        }
    }
}

lazy_static! {
    static ref ASSETS: Assets = Assets::new();
}

struct Assets {
    slots: Slots,
}

impl Assets {
    fn new() -> Assets {
        Assets {
            slots: Slots::new(),
        }
    }
}

struct Slots {
    slots: Mutex<TypeMap>,
}

impl Slots {
    fn new() -> Slots {
        Slots {
            slots: Mutex::new(TypeMap::new()),
        }
    }

    fn acquire<A: Resource>(&self, id: &AssetId) -> Asset<A> {
        let mut type_slots = self.slots.lock().unwrap();
        let slots = type_slots.entry::<AssetTypeMapKey<A>>().or_insert_with(|| HashMap::new());

        if let Some(slot) = slots.get(id) {
            return slot.clone();
        }

        let slot = Asset::new(id.clone());

        slots.insert(id.clone(), slot.clone());

        slot
    }
}

unsafe impl Send for Slots {}
unsafe impl Sync for Slots {}

struct AssetTypeMapKey<A> {
    phantom: PhantomData<A>,
}

impl<A: Resource> Key for AssetTypeMapKey<A> {
    type Value = HashMap<AssetId, Asset<A>>;
}
