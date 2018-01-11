use core::marker::PhantomData;

pub type PhantomLifetime<'a> = PhantomData<&'a ()>;
