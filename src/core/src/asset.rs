use std::path::Path;
use std::collections::HashMap;
use std::sync::RwLock;
use std::ffi::CString;
use std::ptr;
use std::fmt;
use std::sync::Arc;

use Error;

use util::stb_image::*;
use util::cstr_to_string;

pub type AssetID = String;

pub trait AsAssetID {
    fn as_asset_id(&self) -> AssetID;
}

impl<'a> AsAssetID for &'a str {
    fn as_asset_id(&self) -> AssetID {
        self.to_string()
    }
}

pub fn texture<ID: AsAssetID>(id: ID) -> Texture {
    let id = id.as_asset_id();
    Texture {
        slot: ASSETS.acquire_slot(&id),
        id: id,
    }
}

#[derive(Clone)]
pub struct Texture {
    id: AssetID,
    slot: SlotHandle,
}

impl Texture {
    pub fn load<P: AsRef<Path>>(self, path: P) -> Texture {
        if !ASSETS.load_with(&self.id, || {
            unsafe {
                let cstr = CString::new(&*path.as_ref().as_os_str().to_string_lossy()).unwrap();
                let mut w = 0;
                let mut h = 0;
                let data = stbi_load(cstr.as_ptr(), &mut w, &mut h, ptr::null_mut(), 4);
                if data != ptr::null_mut() {
                    info!("Loaded texture `{}` into `{}`.", path.as_ref().display(), self.id);

                    SlotState::Loaded(Box::new(Asset::Texture {
                        w: w,
                        h: h,
                        data: data,
                    }))
                } else {
                    SlotState::LoadError(From::from(format!("Failed to load texture {}: {}", path.as_ref().display(), cstr_to_string(stbi_failure_reason()))))
                }
            }
        }) {
            warn!("Tried to load texture `{}` into `{}` which is already occupied.", path.as_ref().display(), self.id);
        }

        self
    }

    pub fn id(&self) -> &AssetID {
        &self.id
    }

    pub fn access<F, T>(&self, f: F) -> Option<T> where F: FnOnce(i32, i32, *mut u8) -> T {
        let mut result = None;
        ASSETS.with_asset(&self.id, |asset| {
            match *asset {
                Asset::Texture { w, h, data } => {
                    result = Some(f(w, h, data));
                }

                _ => unreachable!(),
            }
        });
        result
    }

    pub fn size(&self) -> (i32, i32) {
        self.access(|w, h, _| {
            (w, h)
        }).unwrap_or((0, 0))
    }
}

impl fmt::Display for Texture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

lazy_static! {
    static ref ASSETS: Assets = Assets::new();
}

struct Assets {
    slots: RwLock<HashMap<AssetID, SlotHandle>>,
}

impl Assets {
    fn new() -> Assets {
        Assets {
            slots: RwLock::new(HashMap::new()),
        }
    }

    fn acquire_slot(&self, id: &AssetID) -> SlotHandle {
        {
            let slots = self.slots.read().unwrap();

            if let Some(slot) = slots.get(id) {
                return slot.clone();
            }
        }

        let mut slots = self.slots.write().unwrap();
        let slot = Arc::new(RwLock::new(Slot {
            state: SlotState::Unloaded,
        }));

        slots.insert(id.clone(), slot.clone());

        slot
    }

    fn with_asset<F>(&self, id: &AssetID, f: F) where F: FnOnce(&Asset) {
        let handle = self.acquire_slot(id);
        let slot = handle.read().unwrap();
        match slot.state {
            SlotState::Loaded(ref asset) => {
                f(&*asset);
            }

            _ => {}
        }
    }

    fn load_with<F>(&self, id: &AssetID, f: F) -> bool where F: FnOnce() -> SlotState {
        let handle = self.acquire_slot(id);

        let load = {
            let mut slot = handle.write().unwrap();
            match slot.state {
                SlotState::Unloaded => {
                    slot.state = SlotState::Loading;
                    true
                }

                _ => false,
            }
        };

        if load {
            let new_state = f();

            let mut slot = handle.write().unwrap();
            slot.state = new_state;
        }

        load
    }
}

enum Asset {
    Texture {
        w: i32,
        h: i32,
        data: *mut u8,
    },

    _Font {},
}

unsafe impl Send for Asset {}
unsafe impl Sync for Asset {}

impl Drop for Asset {
    fn drop(&mut self) {
        match *self {
            Asset::Texture { data, .. } => {
                unsafe { stbi_image_free(data); }
            }

            _ => unreachable!(),
        }
    }
}

enum SlotState {
    Unloaded,
    Loading,
    Loaded(Box<Asset>),
    LoadError(Error),
}

type SlotHandle = Arc<RwLock<Slot>>;

struct Slot {
    state: SlotState,
}
