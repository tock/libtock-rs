use callback::CallbackSubscription;
use callback::SubscribableCallback;
use core::cell::Cell;
use core::isize;
use result;
use result::TockResult;
use result::TockValue;
use syscalls;

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

    let timer = with_callback(|_, _| expired.set(true)).unwrap();

    timer.set_alarm(duration).unwrap();
    syscalls::yieldk_for(|| expired.get());
}

pub fn with_callback<CB: FnMut(ClockValue, Alarm)>(
    callback: CB,
) -> TockResult<Timer<CB>, TimerError> {
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

    let (return_code, subscription) = syscalls::subscribe(TimerCallback {
        callback,
        clock_frequency: ClockFrequency {
            hz: clock_frequency as usize,
        },
    });

    let timer = Timer {
        num_notifications: num_notifications as usize,
        clock_frequency: ClockFrequency {
            hz: clock_frequency as usize,
        },
        subscription,
    };

    match return_code {
        result::SUCCESS => Ok(timer),
        result::ENOMEM => Err(TockValue::Expected(TimerError::SubscriptionFailed)),
        unexpected => Err(TockValue::Unexpected(unexpected)),
    }
}

pub struct Timer<CB: FnMut(ClockValue, Alarm)> {
    num_notifications: usize,
    clock_frequency: ClockFrequency,
    #[allow(dead_code)] // Used in drop
    subscription: CallbackSubscription<TimerCallback<CB>>,
}

#[derive(Copy, Clone, Debug)]
pub enum TimerError {
    NotSupported,
    ErroneousClockFrequency(isize),
    SubscriptionFailed,
}

impl<CB: FnMut(ClockValue, Alarm)> Timer<CB> {
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
            num_ticks: num_ticks,
            clock_frequency: self.clock_frequency,
        }
    }

    pub fn stop_alarm(&self, alarm: Alarm) -> TockResult<(), StopAlarmError> {
        let return_code =
            unsafe { syscalls::command(DRIVER_NUMBER, command_nr::STOP_ALARM, alarm.alarm_id, 0) };

        match return_code {
            result::SUCCESS => Ok(()),
            result::EALREADY => Err(TockValue::Expected(StopAlarmError::AlreadyDisabled)),
            unexpected => Err(TockValue::Unexpected(unexpected)),
        }
    }

    pub fn set_alarm(&self, duration: Duration) -> TockResult<Alarm, SetAlarmError> {
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

struct TimerCallback<CB> {
    callback: CB,
    clock_frequency: ClockFrequency,
}

impl<CB: FnMut(ClockValue, Alarm)> SubscribableCallback for TimerCallback<CB> {
    fn driver_number(&self) -> usize {
        DRIVER_NUMBER
    }

    fn subscribe_number(&self) -> usize {
        subscribe_nr::SUBSCRIBE_CALLBACK
    }

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
