use std::marker::PhantomData;
use super::Act;
use super::internal_iface::ProcessPacketBatch;
use super::packet_batch::cast_from_u8;
use super::TransformBatch;
use super::ReplaceBatch;
use super::super::interface::EndOffset;

pub struct ParsedBatch<'a, T:'a + EndOffset, V> where
    V:'a + ProcessPacketBatch + Act {
    parent: &'a mut V,
    applied: bool,
    phantom: PhantomData<&'a T>,
}

impl<'a, T, V> Act for ParsedBatch<'a, T, V>
    where T:'a + EndOffset,
    V: 'a +  ProcessPacketBatch + Act {
    fn act(&mut self) -> &mut Self {
        self.parent.act();
        self.applied = true;
        self
    }
}

impl<'a, T, V> ParsedBatch<'a, T, V>
    where T: 'a + EndOffset,
    V:'a + ProcessPacketBatch + Act {
    #[inline]
    pub fn new(parent: &'a mut V) -> ParsedBatch<'a, T, V> {
        ParsedBatch{ applied: false, parent: parent, phantom: PhantomData}
    }

    // FIXME: Rename this to something reasonable
    #[inline]
    pub fn parse<T2: EndOffset>(&mut self) -> ParsedBatch<T2, Self> {
        parse!(T2, self)
    }

    #[inline]
    pub fn transform(&'a mut self, transformer: &'a Fn(&mut T)) -> TransformBatch<T, Self> {
        TransformBatch::<T, Self>::new(self, transformer)
    }

    #[inline]
    pub fn pop(&'a mut self) -> &'a mut V {
        self.parent
    }

    #[inline]
    pub fn replace(&'a mut self, template: &'a T) -> ReplaceBatch<T, Self> {
        ReplaceBatch::<T, Self>::new(self, template)
    }
}

impl<'a, T, V> ProcessPacketBatch for ParsedBatch<'a, T, V>
    where T:'a + EndOffset,
    V: 'a +  ProcessPacketBatch + Act {
    #[inline]
    fn start(&self) -> usize {
        self.parent.start()
    }

    #[inline]
    unsafe fn payload(&mut self, idx: usize) -> *mut u8 {
        let address = self.parent.payload(idx);
        let offset = T::offset(cast_from_u8::<T>(address));
        address.offset(offset as isize)
    }

    #[inline]
    unsafe fn address(&mut self, idx: usize) -> *mut u8 {
        self.parent.payload(idx)
    }

    #[inline]
    unsafe fn next_address(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        self.parent.next_payload(idx)
    }

    #[inline]
    unsafe fn next_payload(&mut self, idx: usize) -> Option<(*mut u8, usize)> {
        let parent_payload = self.parent.next_payload(idx);
        match parent_payload {
            Some((packet, idx)) => {
                let offset = T::offset(cast_from_u8::<T>(packet));
                Some((packet.offset(offset as isize), idx))
            }
            None => None
        }
    }
}
