// From: https://github.com/torkleyy/specs-static

use std::hash::Hash;
use std::marker::PhantomData;

use amethyst::ecs::prelude::*;
use amethyst::core::shrev::EventChannel;
use amethyst::ecs::storage::{ComponentEvent, UnprotectedStorage, TryDefault};

type Index = u32;

/// The ids component storages are indexed with. This is mostly just a newtype wrapper with a `u32`
/// in it.
///
/// # Examples
///
/// ```
/// use specs_static::Id;
///
/// #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// pub struct MyAmazingId(pub u32);
///
/// impl Id for MyAmazingId {
///     fn from_u32(value: u32) -> Self {
///         MyAmazingId(value)
///     }
///
///     fn id(&self) -> u32 {
///         self.0
///     }
/// }
/// ```
pub trait Id: Copy + Eq + Hash + Ord + Send + Sync + Sized + std::fmt::Debug + 'static {
    /// Creates an idea from a `u32`.
    fn from_u32(value: u32) -> Self;

    /// Returns the `id` integer value.
    fn id(&self) -> u32;
}

/// A storage for components managed with `specs_static::Id` instead of `Entity`.
/// This `Storage` behaves very similar to `specs`' `Storage`.
///
/// # Registering
///
/// These component storages also have to be registered. This can be done using the `WorldExt`
/// trait and its `register_tile_comp` method.
pub struct Storage<C, D: UnprotectedStorage<C>, I> {
    data: D,
    bitset: BitSet,
    phantom: PhantomData<(C, I)>,
}

impl<C, D: UnprotectedStorage<C>, I> Default for Storage<C, D, I>
    where D: TryDefault
{
    fn default() -> Self {
        Self {
            data: D::unwrap_default(),
            phantom: PhantomData,
            bitset: BitSet::default(),
        }
    }
}

impl<C, D, I> Storage<C, D, I>
    where
        C: Component,
        D: UnprotectedStorage<C>,
        I: Id,
{
    /// Tries to retrieve a component by its `Id`.
    /// This will only check whether a component is inserted or not, without doing
    /// any liveness checks for the id.
    pub fn get(&self, id: I) -> Option<&C> {
        if self.bitset.contains(id.id()) {
            unsafe {
                Some(self.data.get(id.id()))
            }
        } else {
            None
        }
    }

    /// Tries to retrieve a component mutably by its `Id`.
    /// This will only check whether a component is inserted or not, without doing
    /// any liveness checks for the id.
    pub fn get_mut(&mut self, id: I) -> Option<&mut C> {
        if self.bitset.contains(id.id()) {
            unsafe {
                Some(self.data.get_mut(id.id()))
            }
        } else {
            None
        }
    }

    /// Inserts `comp` at `id`. If there already was a value, it will be returned.
    ///
    /// In contrast to entities, **there are no invalid ids.**
    pub fn insert(&mut self, id: I, comp: C) -> Option<C> {
        let old = if self.bitset.add(id.id()) {
            unsafe {
                Some(self.data.remove(id.id()))
            }
        } else {
            None
        };

        unsafe {
            self.data.insert(id.id(), comp);
        }

        old
    }
    pub fn insert_default(&mut self, id: I) -> Option<C>
        where C: Component + Default
    {
        let old = if self.bitset.add(id.id()) {
            unsafe {
                Some(self.data.remove(id.id()))
            }
        } else {
            None
        };

        unsafe {
            self.data.insert(id.id(), C::default());
        }

        old
    }

    /// Removes the component at `id`.
    pub fn remove(&mut self, id: I) -> Option<C> {
        if self.bitset.remove(id.id()) {
            unsafe {
                Some(self.data.remove(id.id()))
            }
        } else {
            None
        }
    }
}

impl<C, D, I> Tracked for Storage<C, D, I>
    where D: Tracked + UnprotectedStorage<C>,
          C: Component
{
    fn channel(&self) -> &EventChannel<ComponentEvent> { self.data.channel() }

    fn channel_mut(&mut self) -> &mut EventChannel<ComponentEvent> { self.data.channel_mut() }
}

impl<C, D, I> Drop for Storage<C, D, I>
    where
        D: UnprotectedStorage<C>,
{
    fn drop(&mut self) {
        unsafe {
            self.data.clean(&self.bitset);
        }
    }
}

impl<'a, C, D, I> Join for &'a Storage<C, D, I>
    where
        D: UnprotectedStorage<C>,
{
    type Type = &'a C;
    type Value = &'a D;
    type Mask = &'a BitSet;

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        (&self.bitset, &self.data)
    }

    unsafe fn get(value: &mut Self::Value, id: Index) -> Self::Type {
        (*value).get(id)
    }
}

impl<'a, C, D, I> Join for &'a mut Storage<C, D, I>
    where
        D: UnprotectedStorage<C>,
{
    type Type = &'a mut C;
    type Value = &'a mut D;
    type Mask = &'a BitSet;

    unsafe fn open(self) -> (Self::Mask, Self::Value) {
        (&self.bitset, &mut self.data)
    }

    unsafe fn get(value: &mut Self::Value, id: Index) -> Self::Type {
        // This is horribly unsafe. Unfortunately, Rust doesn't provide a way
        // to abstract mutable/immutable state at the moment, so we have to hack
        // our way through it.
        #[allow(trivial_casts)]
        let value: *mut Self::Value = value as *mut Self::Value;
        (*value).get_mut(id)
    }
}

/// An extension trait for registering statically managed component storages.
pub trait WorldExt {
    /// Registers a `specs_static::Storage` for the components of type `C`.
    /// This will be done automatically if your storage has a `Default` and you're fetching it with
    /// `Read` / `Write`.
    fn register_tile_comp<C, I>(&mut self)
        where
            C: Component + Send + Sync,
            C::Storage: Default,
            I: Id;
}

impl WorldExt for World {
    fn register_tile_comp<C, I>(&mut self)
        where
            C: Component + Send + Sync,
            C::Storage: Default,
            I: Id,
    {
        self.add_resource(Storage::<C, C::Storage, I>::default());
    }
}