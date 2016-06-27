use std::mem;
use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};

use Error;

use typemap::{TypeMap, Key};

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

pub trait Asset: Any + Clone + Send + Sync {
    fn name() -> &'static str;
}

pub trait Source<A: Asset> {
    fn to_string(&self) -> String;
    fn load(&self) -> Result<A, Error>;
}

enum AssetState<A: Asset> {
    Unloaded,
    Loading,
    Loaded(A),
    LoadError(Error),
}

pub struct Handle<A: Asset> {
    id: AssetId,
    asset: Arc<RwLock<AssetState<A>>>,
}

impl<A: Asset> Handle<A> {
    fn new(id: AssetId) -> Handle<A> {
        Handle {
            id: id,
            asset: Arc::new(RwLock::new(AssetState::Unloaded)),
        }
    }

    pub fn load<S: Source<A>>(&self, src: S) -> Result<(), Error> {
        {
            let mut asset = self.asset.write().unwrap();
            match *asset {
                AssetState::Loading => {
                    return Err(format!("Asset {} is loading.", self).into());
                }

                _ => {
                    *asset = AssetState::Loading;
                }
            }
        }

        let src_string = src.to_string();
        let result = src.load();

        let mut asset = self.asset.write().unwrap();

        match *asset {
            AssetState::Loading => {
                match result {
                    Ok(c) => {
                        *asset = AssetState::Loaded(c);
                        info!("Loaded {} from {}.", self, src_string);
                    }

                    Err(e) => {
                        let err = Err(format!("{}", e).into());
                        *asset = AssetState::LoadError(e);
                        return err;
                    }
                }
            }

            _ => { return Err("Loading interupted".into()) }
        }

        Ok(())
    }

    pub fn id(&self) -> &AssetId {
        &self.id
    }

    pub fn loaded(&self) -> bool {
        match *self.asset.read().unwrap() {
            AssetState::Loaded(_) => true,
            _ => false,
        }
    }

    pub fn get(&self) -> Option<A> {
        let asset = self.asset.read().unwrap();
        match *asset {
            AssetState::Loaded(ref asset) => {
                Some(asset.clone())
            }

            _ => None,
        }
    }

    pub fn set(&self, asset: A) -> Option<A> {
        let mut new_asset = AssetState::Loaded(asset);
        let mut asset = self.asset.write().unwrap();

        mem::swap(&mut *asset, &mut new_asset);

        let old_asset = new_asset;
        match old_asset {
            AssetState::Loaded(asset) => Some(asset),
            _ => None,
        }
    }
}

impl<A: Asset> Clone for Handle<A> {
    fn clone(&self) -> Self {
        Handle {
            id: self.id.clone(),
            asset: self.asset.clone(),
        }
    }
}

impl<A: Asset> fmt::Display for Handle<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", A::name(), self.id())
    }
}

#[allow(non_camel_case_types)]
pub struct asset<A: Asset> {
    phantom: PhantomData<A>,
}

impl<A: Asset> asset<A> {
    pub fn new() -> Handle<A> {
        unimplemented!()
    }

    pub fn with_id<I: ToAssetId>(id: I) -> Handle<A> {
        let id = id.to_asset_id();
        ASSETS.slots.acquire::<A>(id)
    }

    pub fn get<I: ToAssetId>(id: I) -> Option<Handle<A>> {
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

    fn get<A: Asset>(&self, id: &AssetId) -> Option<Handle<A>> {
        let mut type_slots = self.slots.lock().unwrap();
        let slots = type_slots.entry::<AssetTypeMapKey<A>>().or_insert_with(|| HashMap::new());
        slots.get(id).cloned()
    }

    fn acquire<A: Asset>(&self, id: AssetId) -> Handle<A> {
        let mut type_slots = self.slots.lock().unwrap();
        let slots = type_slots.entry::<AssetTypeMapKey<A>>().or_insert_with(|| HashMap::new());

        if let Some(asset) = slots.get(&id) {
            return asset.clone();
        }

        let handle = Handle::new(id.clone());

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
    type Value = HashMap<AssetId, Handle<A>>;
}
