#![expect(missing_docs, reason = "WIP")]

use crate::ptr::Pointee;

#[unstable(feature = "field_projections", issue = "145383")]
pub trait PlaceProxy {
    #[unstable(feature = "field_projections", issue = "145383")]
    type Target: ?Sized;
}

#[unstable(feature = "field_projections", issue = "145383")]
pub trait PlaceHandle: Sized {
    #[unstable(feature = "field_projections", issue = "145383")]
    type Target: ?Sized;
}

#[unstable(feature = "field_projections", issue = "145383")]
pub unsafe trait DerefPlace: PlaceHandle
where
    Self::Target: PlaceProxy,
{
    #[unstable(feature = "field_projections", issue = "145383")]
    type PointeeHandle: PlaceHandle<Target = <Self::Target as PlaceProxy>::Target>;

    #[unstable(feature = "field_projections", issue = "145383")]
    unsafe fn deref_place(self) -> Self::PointeeHandle;
}

#[unstable(feature = "field_projections", issue = "145383")]
pub unsafe trait ProjectPlace<S>: PlaceHandle
where
    S: Subplace<Source = Self::Target>,
{
    #[unstable(feature = "field_projections", issue = "145383")]
    type Projected: PlaceHandle<Target = S::Target>;

    #[unstable(feature = "field_projections", issue = "145383")]
    unsafe fn project_place(self, subplace: S) -> Self::Projected;
}

#[unstable(feature = "field_projections", issue = "145383")]
pub unsafe trait Subplace: Sized {
    #[unstable(feature = "field_projections", issue = "145383")]
    type Source: ?Sized;
    #[unstable(feature = "field_projections", issue = "145383")]
    type Target: ?Sized;

    #[unstable(feature = "field_projections", issue = "145383")]
    fn offset(
        self,
        metadata: <Self::Source as Pointee>::Metadata,
    ) -> (usize, <Self::Target as Pointee>::Metadata);
}

use crate::field::Field;

#[unstable(feature = "field_projections", issue = "145383")]
unsafe impl<F: Field> Subplace for F {
    type Source = F::Base;
    type Target = F::Type;

    fn offset(
        self,
        _metadata: <Self::Source as Pointee>::Metadata,
    ) -> (usize, <Self::Target as Pointee>::Metadata) {
        (F::OFFSET, ())
    }
}

/// Primitive handle referring to a local variable or memory location.
///
/// # Design Tradeoff & Intermediate Status
///
/// `LocalHandle` is an intermediate concrete handle introduced to enable developing,
/// testing, and validating place projections (`ProjectPlace`) and subplaces (`Subplace`)
/// prior to full compiler-native place support.
///
/// In the full Field Projections design:
/// - Place capabilities and lifetimes are statically partitioned into dedicated handle
///   types (e.g. `MutHandle<'a, T>` and `RefHandle<'a, T>`).
/// - Access permissions are checked by the borrow checker through terminal operation
///   traits (`BorrowPlace`, `WritePlace`, `ReadPlace`).
///
/// In this intermediate milestone:
/// - `LocalHandle<T>` acts as a unified concrete handle storing a `*mut T`.
/// - Shared versus exclusive access permissions are not separated into distinct types;
///   callers must uphold `unsafe` preconditions when obtaining or mutating through raw
///   pointers via [`as_mut_ptr`](Self::as_mut_ptr).
#[unstable(feature = "field_projections", issue = "145383")]
pub struct LocalHandle<T: ?Sized> {
    ptr: *mut T,
}

#[unstable(feature = "field_projections", issue = "145383")]
impl<T: ?Sized> Clone for LocalHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

#[unstable(feature = "field_projections", issue = "145383")]
impl<T: ?Sized> Copy for LocalHandle<T> {}

#[unstable(feature = "field_projections", issue = "145383")]
impl<T: ?Sized> crate::fmt::Debug for LocalHandle<T> {
    fn fmt(&self, f: &mut crate::fmt::Formatter<'_>) -> crate::fmt::Result {
        f.debug_struct("LocalHandle").field("ptr", &self.ptr).finish()
    }
}

impl<T: ?Sized> LocalHandle<T> {
    /// Creates a new `LocalHandle` from a raw pointer.
    #[unstable(feature = "field_projections", issue = "145383")]
    pub const unsafe fn new(ptr: *const T) -> Self {
        Self { ptr: ptr as *mut T }
    }

    /// Creates a new mutable `LocalHandle`.
    #[unstable(feature = "field_projections", issue = "145383")]
    pub const unsafe fn new_mut(ptr: *mut T) -> Self {
        Self { ptr }
    }

    /// Obtains a const raw pointer to the place.
    #[unstable(feature = "field_projections", issue = "145383")]
    pub const fn as_ptr(&self) -> *const T {
        self.ptr
    }

    /// Obtains a mutable raw pointer to the place.
    #[unstable(feature = "field_projections", issue = "145383")]
    pub const unsafe fn as_mut_ptr(&self) -> *mut T {
        self.ptr
    }
}

#[unstable(feature = "field_projections", issue = "145383")]
impl<T: ?Sized> PlaceHandle for LocalHandle<T> {
    type Target = T;
}

#[unstable(feature = "field_projections", issue = "145383")]
unsafe impl<S> ProjectPlace<S> for LocalHandle<S::Source>
where
    S: Subplace,
    S::Source: Sized,
    S::Target: Sized,
{
    type Projected = LocalHandle<S::Target>;

    unsafe fn project_place(self, subplace: S) -> Self::Projected {
        let (offset, _) = subplace.offset(());
        // SAFETY: `subplace` guarantees that `offset` produces a valid byte offset
        // within the allocation of `self`.
        let new_ptr = unsafe { (self.ptr as *mut u8).add(offset) as *mut S::Target };
        LocalHandle { ptr: new_ptr }
    }
}

#[unstable(feature = "field_projections", issue = "145383")]
impl<T: ?Sized> PlaceProxy for &T {
    type Target = T;
}

#[unstable(feature = "field_projections", issue = "145383")]
impl<T: ?Sized> PlaceProxy for &mut T {
    type Target = T;
}

#[unstable(feature = "field_projections", issue = "145383")]
impl<T: ?Sized> PlaceProxy for *const T {
    type Target = T;
}

#[unstable(feature = "field_projections", issue = "145383")]
impl<T: ?Sized> PlaceProxy for *mut T {
    type Target = T;
}

#[unstable(feature = "field_projections", issue = "145383")]
unsafe impl<T: ?Sized> DerefPlace for LocalHandle<&mut T> {
    type PointeeHandle = LocalHandle<T>;
    unsafe fn deref_place(self) -> Self::PointeeHandle {
        // SAFETY: `self` refers to a valid proxy location containing a pointer.
        let ptr_val = unsafe { *(self.ptr as *const *mut T) };
        LocalHandle { ptr: ptr_val }
    }
}

#[unstable(feature = "field_projections", issue = "145383")]
unsafe impl<T: ?Sized> DerefPlace for LocalHandle<&T> {
    type PointeeHandle = LocalHandle<T>;
    unsafe fn deref_place(self) -> Self::PointeeHandle {
        // SAFETY: `self` refers to a valid proxy location containing a pointer.
        let ptr_val = unsafe { *(self.ptr as *const *const T) };
        LocalHandle { ptr: ptr_val as *mut T }
    }
}

#[unstable(feature = "field_projections", issue = "145383")]
unsafe impl<T: ?Sized> DerefPlace for LocalHandle<*mut T> {
    type PointeeHandle = LocalHandle<T>;
    unsafe fn deref_place(self) -> Self::PointeeHandle {
        // SAFETY: `self` refers to a valid proxy location containing a pointer.
        let ptr_val = unsafe { *(self.ptr as *const *mut T) };
        LocalHandle { ptr: ptr_val }
    }
}

#[unstable(feature = "field_projections", issue = "145383")]
unsafe impl<T: ?Sized> DerefPlace for LocalHandle<*const T> {
    type PointeeHandle = LocalHandle<T>;
    unsafe fn deref_place(self) -> Self::PointeeHandle {
        // SAFETY: `self` refers to a valid proxy location containing a pointer.
        let ptr_val = unsafe { *(self.ptr as *const *const T) };
        LocalHandle { ptr: ptr_val as *mut T }
    }
}
