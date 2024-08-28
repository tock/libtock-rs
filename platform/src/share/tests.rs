use crate::share::{scope, Handle, List};

std::thread_local! {static INSTANCE_COUNT: core::cell::Cell<u64> = const {core::cell::Cell::new(0)}}

// InstanceCounter increments INSTANCE_COUNT when it is constructed and
// decrements INSTANCE_COUNT when it is dropped.
struct InstanceCounter {
    _private: (),
}

impl Default for InstanceCounter {
    fn default() -> Self {
        INSTANCE_COUNT.with(|cell| cell.set(cell.get() + 1));
        Self { _private: () }
    }
}

impl Drop for InstanceCounter {
    fn drop(&mut self) {
        INSTANCE_COUNT.with(|cell| cell.set(cell.get() - 1));
    }
}

impl List for InstanceCounter {}

#[test]
fn list_lifetime() {
    // INSTANCE_COUNT *should* be 0 here, but make sure it's zero in case
    // another test case leaked an instance.
    INSTANCE_COUNT.with(|cell| cell.set(0));

    scope(|_counter: Handle<InstanceCounter>| {
        assert_eq!(INSTANCE_COUNT.with(|cell| cell.get()), 1);
    });

    assert_eq!(INSTANCE_COUNT.with(|cell| cell.get()), 0);
}

// This test will only compile if the correct trait impls exist for tuples.
#[test]
fn tuple_impls() {
    #[allow(clippy::let_unit_value)]
    scope(|list: Handle<()>| {
        let _empty: () = list.split();
    });

    scope(|list: Handle<((),)>| {
        let (_empty_handle,): (Handle<()>,) = list.split();
    });

    scope(|list: Handle<((), InstanceCounter)>| {
        let (_empty, _counter): (Handle<()>, Handle<InstanceCounter>) = list.split();
    });

    // Tests a size-12 tuple with varying List types inside it.
    #[allow(clippy::type_complexity)]
    scope(
        |list: Handle<(
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            (),
            ((), ()),
            InstanceCounter,
        )>| {
            let (
                _empty1,
                _empty2,
                _empty3,
                _empty4,
                _empty5,
                _empty6,
                _empty7,
                _empty8,
                _empty9,
                _empty10,
                pair,
                _counter,
            ): (
                Handle<()>,
                Handle<()>,
                Handle<()>,
                Handle<()>,
                Handle<()>,
                Handle<()>,
                Handle<()>,
                Handle<()>,
                Handle<()>,
                Handle<()>,
                Handle<((), ())>,
                Handle<InstanceCounter>,
            ) = list.split();
            let (_empty11, _empty12): (Handle<()>, Handle<()>) = pair.split();
        },
    );
}
