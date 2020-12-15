use crate::policy::space::Space;
use crate::scheduler::gc_works::*;
use crate::util::constants::*;
use crate::util::metadata::*;
use crate::util::*;
use crate::MMTK;

/// For field writes in HotSpot, we cannot always get the source object pointer and the field address
pub enum WriteTarget {
    Object(ObjectReference),
    Slot(Address),
}

pub trait Barrier: 'static + Send + Sync {
    fn flush(&mut self);
    fn post_write_barrier(&mut self, target: WriteTarget);
}

pub struct NoBarrier;

impl Barrier for NoBarrier {
    fn flush(&mut self) {}
    fn post_write_barrier(&mut self, _target: WriteTarget) {}
}

pub struct ObjectRememberingBarrier<E: ProcessEdgesWork, S: Space<E::VM>> {
    mmtk: &'static MMTK<E::VM>,
    nursery: &'static S,
    modbuf: Vec<ObjectReference>,
}

impl<E: ProcessEdgesWork, S: Space<E::VM>> ObjectRememberingBarrier<E, S> {
    #[allow(unused)]
    pub fn new(mmtk: &'static MMTK<E::VM>, nursery: &'static S) -> Self {
        Self {
            mmtk,
            nursery,
            modbuf: vec![],
        }
    }

    #[inline(always)]
    fn enqueue_node(&mut self, obj: ObjectReference) {
        // println!("Meta word {:?}", BitsReference::of(obj.to_address(), LOG_BYTES_IN_WORD, 0).word());
        let slow = BitsReference::of(obj.to_address(), LOG_BYTES_IN_WORD, 0);
        // unsafe {
        //     assert!(slow >= Address::from_usize(0x200800000000));
        //     assert!(slow < Address::from_usize(0x200800040000));
        // }
        if BitsReference::of(obj.to_address(), LOG_BYTES_IN_WORD, 0).attempt(0b0, 0b1) {
            self.modbuf.push(obj);
            if self.modbuf.len() >= E::CAPACITY {
                self.flush();
            }
        }
    }
}

impl<E: ProcessEdgesWork, S: Space<E::VM>> Barrier for ObjectRememberingBarrier<E, S> {
    #[cold]
    fn flush(&mut self) {
        let mut modbuf = vec![];
        std::mem::swap(&mut modbuf, &mut self.modbuf);
        debug_assert!(
            !self.mmtk.scheduler.final_stage.is_activated(),
            "{:?}",
            self as *const _
        );
        self.mmtk
            .scheduler
            .closure_stage
            .add(ProcessModBuf::<E>::new(modbuf));
    }

    #[inline(always)]
    fn post_write_barrier(&mut self, target: WriteTarget) {
        match target {
            WriteTarget::Object(obj) => {
                if !self.nursery.in_space(obj) {
                    self.enqueue_node(obj);
                }
            }
            _ => unreachable!(),
        }
    }
}