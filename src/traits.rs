use std::mem;

pub trait Referent {
    type Data;
    type Meta: Copy;

    /// Make a reference from its constituent parts.
    unsafe fn assemble(data: *const Self::Data, meta: Self::Meta) -> *const Self;

    unsafe fn assemble_mut(data: *mut Self::Data, meta: Self::Meta) -> *mut Self;

    /// Break a reference down into its constituent parts.
    fn disassemble(data: *const Self) -> (*const Self::Data, Self::Meta);

    fn disassemble_mut(data: *mut Self) -> (*mut Self::Data, Self::Meta);
}
