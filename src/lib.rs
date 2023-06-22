use std::ops::Deref;

pub trait Mutability {
    type Type<'a, T: 'a + ?Sized>: Deref<Target = T>;
    /// Create a reference from a raw pointer.
    /// # Safety
    /// The caller must ensure that the pointer is valid and points to a valid instance of `T`.
    unsafe fn from_ptr<'a, T: 'a + ?Sized>(ptr: *mut T) -> Self::Type<'a, T>;
}

pub struct Ref;

impl Mutability for Ref {
    type Type<'a, T: 'a + ?Sized> = &'a T;
    unsafe fn from_ptr<'a, T: 'a + ?Sized>(ptr: *mut T) -> Self::Type<'a, T> {
        &*ptr
    }
}

pub struct Mut;

impl Mutability for Mut {
    type Type<'a, T: 'a + ?Sized> = &'a mut T;
    unsafe fn from_ptr<'a, T: 'a + ?Sized>(ptr: *mut T) -> Self::Type<'a, T> {
        &mut *ptr
    }
}

pub type As<'a, M, T> = <M as Mutability>::Type<'a, T>;

pub trait AsRef: Deref {
    type Mut: Mutability;
}
impl<'a, T> AsRef for &'a T {
    type Mut = Ref;
}
impl<'a, T> AsRef for &'a mut T {
    type Mut = Mut;
}

pub type SameMut<'a, A, B> = As<'a, <A as AsRef>::Mut, B>;

#[macro_export]
macro_rules! field {
    ($self:ty, $field:ident, $ty:ty) => {
        impl $self {
            pub fn $field<'a, A: $crate::AsRef<Target = Self>>(
                this: A,
            ) -> $crate::SameMut<'a, A, $ty> {
                unsafe {
                    <A::Mut as $crate::Mutability>::from_ptr(
                        (&this.$field) as *const $ty as *mut $ty,
                    )
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    struct Foo {
        items: Vec<i32>,
    }
    field!(Foo, items, Vec<i32>);
    impl Foo {
        pub fn evens<'a, A: AsRef<Target = Self>>(
            this: A,
        ) -> impl Iterator<Item = SameMut<'a, A, i32>>
        where
            SameMut<'a, A, Vec<i32>>: IntoIterator<Item = SameMut<'a, A, i32>>,
        {
            Self::items(this).into_iter().filter(|x| **x % 2 == 0)
        }
    }

    #[test]
    fn it_works() {
        let mut foo = Foo {
            items: vec![1, 2, 3],
        };
        for even in Foo::evens(&mut foo) {
            *even += 1;
        }
        assert_eq!(vec![1, 3, 3], *Foo::items(&foo));
    }
}
