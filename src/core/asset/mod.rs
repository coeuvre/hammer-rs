use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::path::Path;

use Error;

use typemap::{TypeMap, Key};

pub use self::image::{Image, ImageRef, Frame, FrameRef};
pub use self::animation::{Animation, AnimationRef, WrapMode};

pub mod image;
pub mod animation;

pub trait Asset: Any + Send + Sync {
    fn name() -> &'static str;
}

pub trait Loadable: Sized {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
}

pub struct AssetRef<A: Asset> {
    asset: Arc<RwLock<A>>,
}

enum SlotState<A: Asset> {
    Unloaded,
    Loading,
    Loaded(AssetRef<A>),
    LoadError(Error),
}

impl<A: Asset> Clone for AssetRef<A> {
    fn clone(&self) -> AssetRef<A> {
        AssetRef {
            asset: self.asset.clone()
        }
    }
}

impl<A: Asset> AssetRef<A> {
    pub fn new(asset: A) -> AssetRef<A> {
        AssetRef {
            asset: Arc::new(RwLock::new(asset)),
        }
    }

    pub fn read(&self) -> RwLockReadGuard<A> {
        self.asset.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<A> {
        self.asset.write().unwrap()
    }
}

struct Slot<A: Asset> {
    id: String,
    asset: Arc<RwLock<SlotState<A>>>,
}

impl<A: Asset> Slot<A> {
    fn new(id: String) -> Slot<A> {
        Slot {
            id: id,
            asset: Arc::new(RwLock::new(SlotState::Unloaded)),
        }
    }

    pub fn get(&self) -> Option<AssetRef<A>> {
        let asset = self.asset.read().unwrap();
        match *asset {
            SlotState::Loaded(ref asset) => {
                Some(asset.clone())
            }

            _ => None,
        }
    }

    pub fn set(&self, asset: A) -> Option<AssetRef<A>> {
        let mut new_asset = SlotState::Loaded(AssetRef::new(asset));
        let mut asset = self.asset.write().unwrap();

        ::std::mem::swap(&mut *asset, &mut new_asset);

        let old_asset = new_asset;
        match old_asset {
            SlotState::Loaded(asset) => Some(asset),
            _ => None,
        }
    }

    /*
    pub fn loaded(&self) -> bool {
        match *self.asset.read().unwrap() {
            SlotState::Loaded(_) => true,
            _ => false,
        }
    }

    */
}

impl<A: Asset + Loadable> Slot<A> {
    pub fn load<P: AsRef<Path>>(self, path: P) -> Result<AssetRef<A>, Error> {
        {
            let mut asset = self.asset.write().unwrap();
            match *asset {
                SlotState::Loading => {
                    return Err(format!("Asset {} is loading.", self).into());
                }

                SlotState::Loaded(ref asset) => {
                    return Ok(asset.clone());
                }

                _ => {}
            }

            *asset = SlotState::Loading;
        }

        let result = A::load(path);

        {
            let mut asset = self.asset.write().unwrap();

            match *asset {
                SlotState::Loading => {
                    match result {
                        Ok(a) => {
                            let asset_ref = AssetRef { asset: Arc::new(RwLock::new(a)) };
                            *asset = SlotState::Loaded(asset_ref.clone());
                            info!("Loaded {}", self);
                            Ok(asset_ref)
                        }

                        Err(e) => {
                            let err = Err(format!("{}", e).into());
                            *asset = SlotState::LoadError(e);
                            err
                        }
                    }
                }

                _ => Err("Loading interupted".into()),
            }
        }
    }
}

impl<A: Asset> Clone for Slot<A> {
    fn clone(&self) -> Self {
        Slot {
            id: self.id.clone(),
            asset: self.asset.clone(),
        }
    }
}

impl<A: Asset> fmt::Display for Slot<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", A::name(), self.id)
    }
}

#[allow(non_camel_case_types)]
pub struct asset<A: Asset> {
    phantom: PhantomData<A>,
}

impl<A: Asset + Loadable> asset<A> {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<AssetRef<A>, Error> {
        let id = format!("{}", path.as_ref().display()).replace("\\", "/");
        ASSETS.slots.acquire::<A>(&id).load(path)
    }
}

impl<A: Asset> asset<A> {
    pub fn set<S: Into<String>>(id: S, asset: A) -> Option<AssetRef<A>> {
        let id = id.into();
        let slot = ASSETS.slots.acquire::<A>(&id);
        slot.set(asset)
    }

    pub fn get(id: &str) -> Option<AssetRef<A>> {
        ASSETS.slots.get::<A>(id).and_then(|slot| slot.get())
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

    fn get<A: Asset>(&self, id: &str) -> Option<Slot<A>> {
        let mut type_slots = self.slots.lock().unwrap();
        let slots = type_slots.entry::<AssetTypeMapKey<A>>().or_insert_with(|| HashMap::new());
        slots.get(id).cloned()
    }

    fn acquire<A: Asset>(&self, id: &str) -> Slot<A> {
        let mut type_slots = self.slots.lock().unwrap();
        let slots = type_slots.entry::<AssetTypeMapKey<A>>().or_insert_with(|| HashMap::new());

        if let Some(asset) = slots.get(id) {
            return asset.clone();
        }

        let handle = Slot::new(id.to_string());

        slots.insert(id.to_string(), handle.clone());

        handle
    }
}

unsafe impl Send for Slots {}
unsafe impl Sync for Slots {}

struct AssetTypeMapKey<A> {
    phantom: PhantomData<A>,
}

impl<A: Asset> Key for AssetTypeMapKey<A> {
    type Value = HashMap<String, Slot<A>>;
}
