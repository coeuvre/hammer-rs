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

pub struct Texture {
    id: AssetID,
}

impl Texture {
    pub fn load<P: AsRef<Path>>(self, path: P) -> Texture {
        ASSETS.with_slot(&self.id, |slot| {
            slot.load_with(|| {
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
            })
        });

        self
    }

    pub fn size(&self) -> (i32, i32) {
        let mut size = (0, 0);
        ASSETS.with_slot(&self.id, |slot| {
            slot.with_asset(|asset| {
                match *asset {
                    Asset::Texture { w, h, .. } => {
                        size = (w, h);
                    }

                    _ => unreachable!(),
                }
            })
        });

        size
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

    pub fn with_slot<F>(&self, id: &AssetID, f: F) where F: FnOnce(&Slot) {
        let mut slots = self.slots.lock().unwrap();
        let slot = slots.entry(id.to_string()).or_insert(Slot {
            state: Mutex::new(SlotState::Unloaded),
        });
        f(slot);
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

impl Slot {
    pub fn load_with<F>(&self, f: F) where F: FnOnce() -> SlotState {
        let load = {
            let mut state = self.state.lock().unwrap();
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
            let mut state = self.state.lock().unwrap();
            *state = new_state;
        }
    }

    pub fn with_asset<F>(&self, f: F) where F: FnOnce(&Asset) {
        let state = self.state.lock().unwrap();
        match *state {
            SlotState::Loaded(ref asset) => {
                f(&*asset);
            }

            _ => {}
        }
    }
}
