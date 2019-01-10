use crate::callback::CallbackSubscription;
use crate::callback::SubscribableCallback;
use crate::result;
use crate::result::TockResult;
use crate::result::TockValue;
use crate::syscalls;
use core::cell::Cell;
use core::isize;

const DRIVER_NUMBER: usize = 0x00000;

mod command_nr {
    pub const IS_DRIVER_AVAILABLE: usize = 0;
    pub const GET_CLOCK_FREQUENCY: usize = 1;
    pub const GET_CLOCK_VALUE: usize = 2;
    pub const STOP_ALARM: usize = 3;
    pub const SET_ALARM: usize = 4;
}

mod subscribe_nr {
    pub const SUBSCRIBE_CALLBACK: usize = 0;
}

pub fn sleep(duration: Duration) {
    let expired = Cell::new(false);
    let mut with_callback = with_callback(|_, _| expired.set(true));

    let mut timer = with_callback.init().unwrap();
    timer.set_alarm(duration).unwrap();

    syscalls::yieldk_for(|| expired.get());
}

pub fn with_callback<CB>(callback: CB) -> WithCallback<CB> {
    WithCallback {
        callback,
        clock_frequency: ClockFrequency { hz: 0 },
    }
}

pub struct WithCallback<CB> {
    callback: CB,
    clock_frequency: ClockFrequency,
}

impl<CB: FnMut(ClockValue, Alarm)> SubscribableCallback for WithCallback<CB> {
    fn call_rust(&mut self, clock_value: usize, alarm_id: usize, _: usize) {
        (self.callback)(
            ClockValue {
                num_ticks: clock_value as isize,
                clock_frequency: self.clock_frequency,
            },
            Alarm { alarm_id },
        );
    }
}

impl<CB> WithCallback<CB>
where
    Self: SubscribableCallback,
{
    pub fn init(&mut self) -> TockResult<Timer, TimerError> {
        let num_notifications =
            unsafe { syscalls::command(DRIVER_NUMBER, command_nr::IS_DRIVER_AVAILABLE, 0, 0) };

        if num_notifications < 1 {
            return Err(TockValue::Expected(TimerError::NotSupported));
        }

        let clock_frequency =
            unsafe { syscalls::command(DRIVER_NUMBER, command_nr::GET_CLOCK_FREQUENCY, 0, 0) };

        if clock_frequency < 1 {
            return Err(TockValue::Expected(TimerError::ErroneousClockFrequency(
                clock_frequency,
            )));
        }

        let clock_frequency = ClockFrequency {
            hz: clock_frequency as usize,
        };
        self.clock_frequency = clock_frequency;

        let subscription =
            syscalls::subscribe(DRIVER_NUMBER, subscribe_nr::SUBSCRIBE_CALLBACK, self);

        match subscription {
            Ok(subscription) => Ok(Timer {
                num_notifications: num_notifications as usize,
                clock_frequency,
                subscription,
            }),
            Err(result::ENOMEM) => Err(TockValue::Expected(TimerError::SubscriptionFailed)),
            Err(unexpected) => Err(TockValue::Unexpected(unexpected)),
        }
    }
}

pub struct Timer<'a> {
    num_notifications: usize,
    clock_frequency: ClockFrequency,
    #[allow(dead_code)] // Used in drop
    subscription: CallbackSubscription<'a>,
}

#[derive(Copy, Clone, Debug)]
pub enum TimerError {
    NotSupported,
    ErroneousClockFrequency(isize),
    SubscriptionFailed,
}

impl<'a> Timer<'a> {
    pub fn num_notifications(&self) -> usize {
        self.num_notifications
    }

    pub fn clock_frequency(&self) -> ClockFrequency {
        self.clock_frequency
    }

    pub fn get_current_clock(&self) -> ClockValue {
        let num_ticks =
            unsafe { syscalls::command(DRIVER_NUMBER, command_nr::GET_CLOCK_VALUE, 0, 0) };

        ClockValue {
            num_ticks,
            clock_frequency: self.clock_frequency,
        }
    }

    pub fn stop_alarm(&mut self, alarm: Alarm) -> TockResult<(), StopAlarmError> {
        let return_code =
            unsafe { syscalls::command(DRIVER_NUMBER, command_nr::STOP_ALARM, alarm.alarm_id, 0) };

        match return_code {
            result::SUCCESS => Ok(()),
            result::EALREADY => Err(TockValue::Expected(StopAlarmError::AlreadyDisabled)),
            unexpected => Err(TockValue::Unexpected(unexpected)),
        }
    }

    pub fn set_alarm(&mut self, duration: Duration) -> TockResult<Alarm, SetAlarmError> {
        let now = self.get_current_clock();
        let alarm_instant =
            now.num_ticks() as usize + (duration.ms() as usize * self.clock_frequency.hz()) / 1000;

        let alarm_id =
            unsafe { syscalls::command(DRIVER_NUMBER, command_nr::SET_ALARM, alarm_instant, 0) };

        match alarm_id {
            _ if alarm_id >= 0 => Ok(Alarm {
                alarm_id: alarm_id as usize,
            }),
            result::ENOMEM => Err(TockValue::Expected(SetAlarmError::NoMemoryAvailable)),
            unexpected => Err(TockValue::Unexpected(unexpected)),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ClockFrequency {
    hz: usize,
}

impl ClockFrequency {
    pub fn hz(&self) -> usize {
        self.hz
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ClockValue {
    num_ticks: isize,
    clock_frequency: ClockFrequency,
}

impl ClockValue {
    pub fn num_ticks(&self) -> isize {
        self.num_ticks
    }

    pub fn ms(&self) -> isize {
        if self.num_ticks.abs() < isize::MAX / 1000 {
            (1000 * self.num_ticks) / self.clock_frequency.hz() as isize
        } else {
            1000 * (self.num_ticks / self.clock_frequency.hz() as isize)
        }
    }
}

pub struct Alarm {
    alarm_id: usize,
}

impl Alarm {
    pub fn alarm_id(&self) -> usize {
        self.alarm_id
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StopAlarmError {
    AlreadyDisabled,
}

#[derive(Clone, Copy, Debug)]
pub enum SetAlarmError {
    NoMemoryAvailable,
}

#[derive(Copy, Clone, Debug)]
pub struct Duration {
    ms: isize,
}

impl Duration {
    pub fn from_ms(ms: isize) -> Duration {
        Duration { ms }
    }

    pub fn ms(&self) -> isize {
        self.ms
    }
}
