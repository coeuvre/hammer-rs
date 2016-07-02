use std::mem;
use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

use Error;

use typemap::{TypeMap, Key};

pub use self::image::{Image, ImageRef};
// pub use self::sprite::{Sprite, SpriteRef};

pub mod image;
pub mod sprite;

pub type AssetId = String;

pub trait ToAssetId {
    fn to_asset_id(&self) -> AssetId;
}

impl<'a> ToAssetId for &'a str {
    fn to_asset_id(&self) -> AssetId {
        ::std::string::ToString::to_string(self)
    }
}

pub trait Asset: Any + Send + Sync {
    fn name() -> &'static str;
}

pub trait Source<A: Asset> {
    fn to_string(&self) -> String;
    fn load(&self) -> Result<A, Error>;
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
    pub fn read(&self) -> RwLockReadGuard<A> {
        self.asset.read().unwrap()
    }
}

pub struct Slot<A: Asset> {
    id: AssetId,
    asset: Arc<RwLock<SlotState<A>>>,
}

impl<A: Asset> Slot<A> {
    fn new(id: AssetId) -> Slot<A> {
        Slot {
            id: id,
            asset: Arc::new(RwLock::new(SlotState::Unloaded)),
        }
    }

    pub fn load<S: Source<A>>(self, src: S) -> Result<AssetRef<A>, Error> {
        {
            let mut asset = self.asset.write().unwrap();
            match *asset {
                SlotState::Loading => {
                    return Err(format!("Asset {} is loading.", self).into());
                }

                _ => {
                    *asset = SlotState::Loading;
                }
            }
        }

        let src_string = src.to_string();
        let result = src.load();

        let mut asset = self.asset.write().unwrap();

        match *asset {
            SlotState::Loading => {
                match result {
                    Ok(a) => {
                        let asset_ref = AssetRef { asset: Arc::new(RwLock::new(a)) };
                        *asset = SlotState::Loaded(asset_ref.clone());
                        info!("Loaded {} from {}.", self, src_string);
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

    pub fn id(&self) -> &AssetId {
        &self.id
    }

    pub fn loaded(&self) -> bool {
        match *self.asset.read().unwrap() {
            SlotState::Loaded(_) => true,
            _ => false,
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

    pub fn set(&self, asset: AssetRef<A>) -> Option<AssetRef<A>> {
        let mut new_asset = SlotState::Loaded(asset);
        let mut asset = self.asset.write().unwrap();

        mem::swap(&mut *asset, &mut new_asset);

        let old_asset = new_asset;
        match old_asset {
            SlotState::Loaded(asset) => Some(asset),
            _ => None,
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
        write!(f, "{}({})", A::name(), self.id())
    }
}

#[allow(non_camel_case_types)]
pub struct asset<A: Asset> {
    phantom: PhantomData<A>,
}

use util::counter::Counter;
lazy_static! {
    static ref COUNTER: Counter = Counter::new();
}

impl<A: Asset> asset<A> {
    pub fn new() -> Slot<A> {
        let id = format!("{} {}", A::name(), COUNTER.next());
        asset::with_id(id.as_str())
    }

    pub fn with_id<I: ToAssetId>(id: I) -> Slot<A> {
        let id = id.to_asset_id();
        ASSETS.slots.acquire::<A>(id)
    }

    pub fn get<I: ToAssetId>(id: I) -> Option<Slot<A>> {
        let id = id.to_asset_id();
        ASSETS.slots.get::<A>(&id)
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

    fn get<A: Asset>(&self, id: &AssetId) -> Option<Slot<A>> {
        let mut type_slots = self.slots.lock().unwrap();
        let slots = type_slots.entry::<AssetTypeMapKey<A>>().or_insert_with(|| HashMap::new());
        slots.get(id).cloned()
    }

    fn acquire<A: Asset>(&self, id: AssetId) -> Slot<A> {
        let mut type_slots = self.slots.lock().unwrap();
        let slots = type_slots.entry::<AssetTypeMapKey<A>>().or_insert_with(|| HashMap::new());

        if let Some(asset) = slots.get(&id) {
            return asset.clone();
        }

        let handle = Slot::new(id.clone());

        slots.insert(id, handle.clone());

        handle
    }
}

unsafe impl Send for Slots {}
unsafe impl Sync for Slots {}

struct AssetTypeMapKey<A> {
    phantom: PhantomData<A>,
}

impl<A: Asset> Key for AssetTypeMapKey<A> {
    type Value = HashMap<AssetId, Slot<A>>;
}
