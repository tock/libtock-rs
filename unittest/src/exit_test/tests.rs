use super::*;

#[test]
fn exitcall_display() {
    assert_eq!(format!("{}", ExitCall::Terminate(3)), "exit-terminate(3)");
    assert_eq!(format!("{}", ExitCall::Restart(14)), "exit-restart(14)");
}

#[test]
fn exitcall_parse() {
    assert_eq!("exit-terminate(3)".parse(), Ok(ExitCall::Terminate(3)));
    assert_eq!("exit-restart(14)".parse(), Ok(ExitCall::Restart(14)));
    assert_eq!("exit-unknown(3)".parse::<ExitCall>(), Err(ParseExitError));
    assert_eq!(
        "exit-restart(not-an-int)".parse::<ExitCall>(),
        Err(ParseExitError)
    );
    assert_eq!("no-parens".parse::<ExitCall>(), Err(ParseExitError));
    assert_eq!("".parse::<ExitCall>(), Err(ParseExitError));
}

#[test]
fn exitmessage_display() {
    assert_eq!(
        format!("{}", ExitMessage::ExitCall(ExitCall::Restart(1))),
        "ExitCall(exit-restart(1))"
    );
    assert_eq!(format!("{}", ExitMessage::WrongCase), "WrongCase");
    assert_eq!(format!("{}", ExitMessage::DidNotExit), "DidNotExit");
}

#[test]
fn exitmessage_parse() {
    assert_eq!("".parse::<ExitMessage>(), Err(ParseExitError));
    assert_eq!("ExitCall()".parse::<ExitMessage>(), Err(ParseExitError));
    assert_eq!(
        "ExitCall(error)".parse::<ExitMessage>(),
        Err(ParseExitError)
    );
    assert_eq!(
        "ExitCall(exit-restart(5))".parse::<ExitMessage>(),
        Ok(ExitMessage::ExitCall(ExitCall::Restart(5)))
    );
    assert_eq!(
        "WrongCase".parse::<ExitMessage>(),
        Ok(ExitMessage::WrongCase)
    );
    assert_eq!(
        "DidNotExit".parse::<ExitMessage>(),
        Ok(ExitMessage::DidNotExit)
    );
}

#[should_panic(expected = "did not call Exit")]
#[test]
fn exit_test_did_not_exit() {
    exit_test("exit_test::tests::exit_test_did_not_exit", || {});
}

#[should_panic(expected = "did not indicate why it exited")]
#[test]
fn exit_test_did_not_signal() {
    exit_test("exit_test::tests::exit_test_did_not_signal", || {
        std::process::exit(1)
    });
}

#[test]
fn exit_test_signal_terminate() {
    let result = exit_test("exit_test::tests::exit_test_signal_terminate", || {
        signal_exit(ExitCall::Terminate(159));
        std::process::exit(1);
    });
    assert_eq!(result, ExitCall::Terminate(159));
}

#[test]
fn exit_test_signal_restart() {
    let result = exit_test("exit_test::tests::exit_test_signal_restart", || {
        signal_exit(ExitCall::Restart(0));
        std::process::exit(1);
    });
    assert_eq!(result, ExitCall::Restart(0));
}

#[should_panic(expected = "executed the wrong test case")]
#[test]
fn exit_test_wrong_case() {
    // Intentionally-incorrect test case name.
    exit_test("exit_test::tests::exit_test_signal_restart", || {
        signal_exit(ExitCall::Restart(0));
        std::process::exit(1);
    });
}
