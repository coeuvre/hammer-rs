use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::fmt;
use std::path::Path;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::{Mutex, RwLock, RwLockReadGuard};

use Error;

use typemap::{TypeMap, Key};

pub mod image;

pub type AssetId = String;

pub trait AsAssetId {
    fn as_asset_id(&self) -> AssetId;
}

impl<'a> AsAssetId for &'a str {
    fn as_asset_id(&self) -> AssetId {
        self.to_string()
    }
}

pub trait Resource: Any + Send {
    fn type_name() -> &'static str;
}

pub trait Loadable: Sized {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error>;
}

enum AssetState<A> {
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
    pub fn id(&self) -> &AssetId {
        &self.id
    }

    pub fn loaded(&self) -> bool {
        match *self.state.read().unwrap() {
            AssetState::Loaded(_) => true,
            _ => false,
        }
    }

    pub fn borrow(&self) -> Option<AssetRef<A>> {
        // Panic if can't read
        let guard = self.state.try_read().unwrap();
        match *guard {
            AssetState::Loaded(_) => {
                Some(AssetRef {
                    asset: self,
                    guard: guard,
                })
            }

            _ => None,
        }
    }
}

impl<A: Resource> Asset<A> {
    pub fn load_with<F: FnOnce() -> Result<A, Error>>(&self, f: F) -> Result<(), Error> {
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
                    err = Some(format!("{}", e));
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

    pub fn access<T, F: FnOnce(AssetRef<A>) -> T>(&self, f: F) -> Result<T, Error> {
        match self.borrow() {
            Some(asset) => Ok(f(asset)),
            None => Err(format!("Failed to access {}", self).into()),
        }
    }
}

impl<A: Resource + Loadable> Asset<A> {
    pub fn load<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        self.load_with(|| {
            let result = A::load(path.as_ref());
            if result.is_ok() {
                info!("Loaded {} from {}.", self, path.as_ref().display());
            }
            result
        })
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

impl<A: Resource> fmt::Display for Asset<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", A::type_name(), self.id())
    }
}

pub struct AssetRef<'a, A: 'a> {
    asset: &'a Asset<A>,
    guard: RwLockReadGuard<'a, AssetState<A>>,
}

impl<'a, A: 'a> AssetRef<'a, A> {
    pub fn id(&self) -> &AssetId {
        self.asset.id()
    }
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

        if let Some(asset) = slots.get(id) {
            return asset.clone();
        }

        let asset = Asset {
            id: id.clone(),
            state: Arc::new(RwLock::new(AssetState::Unloaded)),
        };

        slots.insert(id.clone(), asset.clone());

        asset
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
