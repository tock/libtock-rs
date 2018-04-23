use callback::CallbackSubscription;
use callback::SubscribableCallback;
use result::TockError;
use result::TockResult;
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

pub fn with_callback<CB>(callback: CB) -> WithCallback<CB> {
    WithCallback { callback }
}

pub struct WithCallback<CB> {
    callback: CB,
}

impl<CB: FnMut(usize, ButtonState)> SubscribableCallback for WithCallback<CB> {
    fn call_rust(&mut self, button_num: usize, state: usize, _: usize) {
        (self.callback)(button_num, state.into());
    }
}

impl<CB> WithCallback<CB>
where
    Self: SubscribableCallback,
{
    pub fn init(&mut self) -> Result<Buttons, ButtonsError> {
        let count = unsafe { syscalls::command(DRIVER_NUMBER, command_nr::COUNT, 0, 0) }
            .map_err(ButtonsError::GetDriverFailed)?;

        if count == 0 {
            return Err(ButtonsError::NotSupported);
        }

        let subscription =
            syscalls::subscribe(DRIVER_NUMBER, subscribe_nr::SUBSCRIBE_CALLBACK, self)
                .map_err(ButtonsError::SubscriptionFailed)?;

        Ok(Buttons {
            count,
            subscription,
        })
    }
}

pub struct Buttons<'a> {
    count: usize,
    #[allow(dead_code)] // Used in drop
    subscription: CallbackSubscription<'a>,
}

#[derive(Copy, Clone, Debug)]
pub enum ButtonsError {
    GetDriverFailed(TockError),
    NotSupported,
    SubscriptionFailed(TockError),
}

impl<'a> Buttons<'a> {
    pub fn iter_mut(&mut self) -> ButtonIter {
        ButtonIter {
            curr_button: 0,
            button_count: self.count,
            _lifetime: &(),
        }
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

impl<'a, 'b> IntoIterator for &'b mut Buttons<'a> {
    type Item = ButtonHandle<'b>;
    type IntoIter = ButtonIter<'b>;

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
    pub fn enable(&mut self) -> TockResult<Button> {
        unsafe {
            syscalls::command(
                DRIVER_NUMBER,
                command_nr::ENABLE_INTERRUPT,
                self.button_num,
                0,
            )
        }?;

        Ok(Button { handle: self })
    }

    pub fn disable(&mut self) -> TockResult<()> {
        unsafe {
            syscalls::command(
                DRIVER_NUMBER,
                command_nr::DISABLE_INTERRUPT,
                self.button_num,
                0,
            )
        }?;

        Ok(())
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
    pub fn read(&self) -> TockResult<ButtonState> {
        let button_state = unsafe {
            syscalls::command(DRIVER_NUMBER, command_nr::READ, self.handle.button_num, 0)
        }?;

        Ok(ButtonState::from(button_state))
    }
}
