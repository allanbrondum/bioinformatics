use alloc::rc::Rc;
use std::ops::Deref;
use bumpalo::Bump;


pub trait ReferencingAllocator {
    type Ref<T:'static>: Deref<Target = T> + Clone;

    fn allocate_referenced<T:'static>(&self, val: T) -> Self::Ref<T>;
}

#[derive(Clone, Copy)]
pub struct StdAllocator<A: std::alloc::Allocator>(pub A);

impl<A: std::alloc::Allocator + Copy> ReferencingAllocator for StdAllocator<A> {
    type Ref<T:'static> = Rc<T, A>;

    fn allocate_referenced<T:'static>(&self, val: T) -> Self::Ref<T> {
        Rc::new_in(val, self.0)
    }
}

#[derive(Clone, Copy)]
pub struct BumpAllocator<'bump>(pub &'bump Bump);


impl<'bump> ReferencingAllocator for BumpAllocator<'bump> {
    type Ref<T:'static> = &'bump T;

    fn allocate_referenced<T:'static>(&self, val: T) -> Self::Ref<T> {
        self.0.alloc(val)
    }
}


