use ::policy::immortalspace::ImmortalSpace;
use ::util::alloc::BumpAllocator;
use ::plan::mutator_context::MutatorContext;
use ::plan::Phase;
use ::util::Address;
use ::util::alloc::Allocator;
use ::plan::Allocator as AllocationType;

#[repr(C)]
pub struct NoGCMutator<'a> {
    // ImmortalLocal
    nogc: BumpAllocator<'a, ImmortalSpace>
}

impl<'a> MutatorContext for NoGCMutator<'a> {
    fn collection_phase(&mut self, thread_id: usize, phase: &Phase, primary: bool) {
        unimplemented!();
    }

    fn alloc(&mut self, size: usize, align: usize, offset: isize, allocator: AllocationType) -> Address {
        trace!("MutatorContext.alloc({}, {}, {}, {:?})", size, align, offset, allocator);
        self.nogc.alloc(size, align, offset)
    }

    fn alloc_slow(&mut self, size: usize, align: usize, offset: isize, allocator: AllocationType) -> Address {
        trace!("MutatorContext.alloc_slow({}, {}, {}, {:?})", size, align, offset, allocator);
        self.nogc.alloc_slow(size, align, offset)
    }
    fn get_thread_id(&self) -> usize {
        self.nogc.thread_id
    }
}

impl<'a> NoGCMutator<'a> {
    pub fn new(thread_id: usize, space: &'a ImmortalSpace) -> Self {
        NoGCMutator {
            nogc: BumpAllocator::new(thread_id, Some(space)),
        }
    }
}