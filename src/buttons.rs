use callback::CallbackSubscription;
use callback::SubscribableCallback;
use result;
use result::TockResult;
use result::TockValue;
use syscalls;

const DRIVER_NUMBER: usize = 0x00003;

mod command_nr {
    pub const COUNT: usize = 0;
    pub const ENABLE_INTERRUPT: usize = 1;
    pub const DISABLE_INTERRUPT: usize = 2;
    pub const READ: usize = 3;
}

mod subscribe_nr {
    pub const SUBSCRIBE_CALLBACK: usize = 0;
}

pub fn with_callback<CB: FnMut(usize, ButtonState)>(
    callback: CB,
) -> TockResult<Buttons<CB>, ButtonsError> {
    let count = unsafe { syscalls::command(DRIVER_NUMBER, command_nr::COUNT, 0, 0) };

    if count < 1 {
        return Err(TockValue::Expected(ButtonsError::NotSupported));
    }

    let (return_code, subscription) = syscalls::subscribe_new(ButtonsCallback { callback });

    match return_code {
        result::SUCCESS => Ok(Buttons {
            count: count as usize,
            subscription,
        }),
        result::ENOMEM => Err(TockValue::Expected(ButtonsError::SubscriptionFailed)),
        unexpected => Err(TockValue::Unexpected(unexpected)),
    }
}

pub struct Buttons<CB: FnMut(usize, ButtonState)> {
    count: usize,
    #[allow(dead_code)] // Used in drop
    subscription: CallbackSubscription<ButtonsCallback<CB>>,
}

#[derive(Copy, Clone, Debug)]
pub enum ButtonsError {
    NotSupported,
    SubscriptionFailed,
}

impl<CB: FnMut(usize, ButtonState)> Buttons<CB> {
    pub fn iter_mut(&mut self) -> ButtonIter {
        ButtonIter {
            curr_button: 0,
            button_count: self.count,
            _lifetime: &(),
        }
    }
}

struct ButtonsCallback<CB> {
    callback: CB,
}

impl<CB: FnMut(usize, ButtonState)> SubscribableCallback for ButtonsCallback<CB> {
    fn driver_number(&self) -> usize {
        DRIVER_NUMBER
    }

    fn subscribe_number(&self) -> usize {
        subscribe_nr::SUBSCRIBE_CALLBACK
    }

    fn call_rust(&mut self, button_num: usize, state: usize, _: usize) {
        (self.callback)(button_num, state.into());
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ButtonState {
    Pressed,
    Released,
}

impl From<usize> for ButtonState {
    fn from(state: usize) -> ButtonState {
        match state {
            0 => ButtonState::Released,
            1 => ButtonState::Pressed,
            _ => unreachable!(),
        }
    }
}

impl<'a, CB: FnMut(usize, ButtonState)> IntoIterator for &'a mut Buttons<CB> {
    type Item = ButtonHandle<'a>;
    type IntoIter = ButtonIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct ButtonIter<'a> {
    curr_button: usize,
    button_count: usize,
    _lifetime: &'a (),
}

impl<'a> Iterator for ButtonIter<'a> {
    type Item = ButtonHandle<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_button < self.button_count {
            let item = ButtonHandle {
                button_num: self.curr_button,
                _lifetime: &(),
            };
            self.curr_button += 1;
            Some(item)
        } else {
            None
        }
    }
}

pub struct ButtonHandle<'a> {
    button_num: usize,
    _lifetime: &'a (),
}

impl<'a> ButtonHandle<'a> {
    pub fn enable(&mut self) -> TockResult<Button, ButtonError> {
        let return_code = unsafe {
            syscalls::command(
                DRIVER_NUMBER,
                command_nr::ENABLE_INTERRUPT,
                self.button_num,
                0,
            )
        };

        match return_code {
            result::SUCCESS => Ok(Button { handle: self }),
            result::ENOMEM => Err(TockValue::Expected(ButtonError::ActivationFailed)),
            unexpected => Err(TockValue::Unexpected(unexpected)),
        }
    }

    pub fn disable(&mut self) -> TockResult<(), ButtonError> {
        let return_code = unsafe {
            syscalls::command(
                DRIVER_NUMBER,
                command_nr::DISABLE_INTERRUPT,
                self.button_num,
                0,
            )
        };

        match return_code {
            result::SUCCESS => Ok(()),
            result::ENOMEM => Err(TockValue::Expected(ButtonError::ActivationFailed)),
            unexpected => Err(TockValue::Unexpected(unexpected)),
        }
    }
}

pub struct Button<'a> {
    handle: &'a ButtonHandle<'a>,
}

#[derive(Copy, Clone, Debug)]
pub enum ButtonError {
    ActivationFailed,
}

impl<'a> Button<'a> {
    pub fn read(&self) -> ButtonState {
        unsafe {
            ButtonState::from(syscalls::command(
                DRIVER_NUMBER,
                command_nr::READ,
                self.handle.button_num,
                0,
            ) as usize)
        }
    }
}
