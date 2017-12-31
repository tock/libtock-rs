/// Lang item required to make the normal `main` work in applications
// This is how the `start` lang item works:
// When `rustc` compiles a binary crate, it creates a `main` function that looks
// like this:
//
// ```
// #[export_name = "main"]
// pub extern "C" fn rustc_main(argc: isize, argv: *const *const u8) -> isize {
//     start(main)
// }
// ```
//
// Where `start` is this function and `main` is the binary crate's `main`
// function.
//
// The final piece is that the entry point of our program, _start, has to call
// `rustc_main`. That's covered by the `_start` function in the root of this
// crate.
use led;
use timer;
#[lang = "start"]
extern "C" fn start(main: fn(), _argc: isize, _argv: *const *const u8) -> isize {
    main();

    0
}

#[lang = "eh_personality"]
extern "C" fn eh_personality() {
    circle_leds(); // TODO: Make NRF52dK specific
}


#[lang = "panic_fmt"]
unsafe extern "C" fn rust_begin_unwind() {
    flash_all_leds(); // TODO: Make NRF52DK specific
}

fn circle_leds() {
    loop {
        led::on(0);
        timer::delay_ms(100);
        led::off(0);
        led::on(1);
        timer::delay_ms(100);
        led::off(1);
        led::on(3);
        timer::delay_ms(100);
        led::off(3);
        led::on(2);
        timer::delay_ms(100);
        led::off(2);
    }
}

fn flash_all_leds() {
    loop {
        led::on(0);
        led::on(1);
        led::on(2);
        led::on(3);
        timer::delay_ms(100);
        led::off(0);
        led::off(1);
        led::off(2);
        led::off(3);
        timer::delay_ms(100);
    }
}
