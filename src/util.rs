use core::marker::PhantomData;

pub(crate) type PhantomLifetime<'a> = PhantomData<&'a ()>;
