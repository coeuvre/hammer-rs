use std::path::Path;
use std::collections::HashMap;
use std::sync::Mutex;
use std::ffi::CString;
use std::ptr;

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
    Texture {
        id: id.as_asset_id(),
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Texture {
    id: AssetID,
}

impl Texture {
    pub fn load<P: AsRef<Path>>(self, path: P) -> Texture {
        ASSETS.load_with(&self.id, || {
            unsafe {
                let cstr = CString::new(&*path.as_ref().as_os_str().to_string_lossy()).unwrap();
                let mut w = 0;
                let mut h = 0;
                let data = stbi_load(cstr.as_ptr(), &mut w, &mut h, ptr::null_mut(), 4);
                if data != ptr::null_mut() {
                    info!("Loaded texture {} ({}x{})", path.as_ref().display(), w, h);

                    SlotState::Loaded(Box::new(Asset::Texture {
                        w: w,
                        h: h,
                        data: data,
                    }))
                } else {
                    SlotState::LoadError(From::from(format!("Failed to load texture {}: {}", path.as_ref().display(), cstr_to_string(stbi_failure_reason()))))
                }
            }
        });

        self
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

lazy_static! {
    static ref ASSETS: Assets = Assets::new();
}

struct Assets {
    slots: Mutex<HashMap<AssetID, Slot>>,
}

impl Assets {
    pub fn new() -> Assets {
        Assets {
            slots: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_asset<F>(&self, id: &AssetID, f: F) where F: FnOnce(&Asset) {
        let mut slots = self.slots.lock().unwrap();
        let slot = slots.entry(id.clone()).or_insert(Slot {
            state: Mutex::new(SlotState::Unloaded),
        });
        let state = slot.state.lock().unwrap();
        match *state {
            SlotState::Loaded(ref asset) => {
                f(&*asset);
            }

            _ => {}
        }
    }

    pub fn load_with<F>(&self, id: &AssetID, f: F) where F: FnOnce() -> SlotState {
        let load = {
            let mut slots = self.slots.lock().unwrap();
            let slot = slots.entry(id.clone()).or_insert(Slot {
                state: Mutex::new(SlotState::Unloaded),
            });
            let mut state = slot.state.lock().unwrap();
            match *state {
                SlotState::Unloaded => {
                    *state = SlotState::Loading;
                    true
                }

                _ => false,
            }
        };

        if load {
            let new_state = f();

            let slots = self.slots.lock().unwrap();
            let slot = slots.get(id).unwrap();
            let mut state = slot.state.lock().unwrap();
            *state = new_state
        }
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

struct Slot {
    state: Mutex<SlotState>,
}
