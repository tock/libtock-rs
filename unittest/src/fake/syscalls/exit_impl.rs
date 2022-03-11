use core::convert::TryInto;

pub(super) fn exit(r0: libtock_platform::Register, r1: libtock_platform::Register) -> ! {
    let exit_num: u32 = r0.try_into().expect("Too large exit number");
    let completion_code: u32 = r1.try_into().expect("Too large completion code");
    match exit_num {
        libtock_platform::exit_id::TERMINATE => {
            println!("exit-terminate called with code {}", completion_code);

            #[cfg(not(miri))]
            crate::exit_test::signal_exit(crate::ExitCall::Terminate(completion_code));

            std::process::exit(1);
        }
        libtock_platform::exit_id::RESTART => {
            println!("exit-restart called with code {}", completion_code);

            #[cfg(not(miri))]
            crate::exit_test::signal_exit(crate::ExitCall::Restart(completion_code));

            std::process::exit(1);
        }
        _ => panic!("Unknown exit number {} invoked.", exit_num),
    }
}
