use crate::adc::AdcDriverFactory;
use crate::buttons::ButtonsDriverFactory;
use crate::console::ConsoleDriver;
use crate::ctap::CtapDriverFactory;
use crate::gpio::GpioDriverFactory;
use crate::hmac::HmacDriverFactory;
use crate::i2c_master_slave::I2cMSDriverFactory;
use crate::leds::LedsDriverFactory;
use crate::result::OtherError;
use crate::result::TockError;
use crate::rng::RngDriver;
use crate::sensors::ninedof::NinedofDriver;
use crate::sensors::AmbientLightSensor;
use crate::sensors::HumiditySensor;
use crate::sensors::TemperatureSensor;
use crate::simple_ble::BleAdvertisingDriverFactory;
use crate::simple_ble::BleScanningDriverFactory;
use crate::temperature::TemperatureDriverFactory;
use crate::timer::DriverContext;
use core::cell::Cell;

/// Struct containing all drivers constructible through [retrieve_drivers()]
#[non_exhaustive]
pub struct Drivers {
    pub console: ConsoleDriver,
    pub ctap: CtapDriverFactory,
    pub leds: LedsDriverFactory,
    pub timer: DriverContext,
    pub gpio: GpioDriverFactory,
    pub hmac: HmacDriverFactory,
    pub temperature: TemperatureDriverFactory,
    pub buttons: ButtonsDriverFactory,
    pub adc: AdcDriverFactory,
    pub i2c_ms: I2cMSDriverFactory,
    pub rng: RngDriver,
    pub ble_advertising: BleAdvertisingDriverFactory,
    pub ble_scanning: BleScanningDriverFactory,
    pub ambient_light_sensor: AmbientLightSensor,
    pub temperature_sensor: TemperatureSensor,
    pub humidity_sensor: HumiditySensor,
    pub ninedof: NinedofDriver,
}

/// Retrieve [Drivers] struct. Returns struct only once.
pub fn retrieve_drivers() -> Result<Drivers, DriversAlreadyTakenError> {
    static mut DRIVER_TAKEN: bool = false;

    unsafe {
        if DRIVER_TAKEN {
            Err(DriversAlreadyTakenError)
        } else {
            DRIVER_TAKEN = true;
            Ok(retrieve_drivers_unsafe())
        }
    }
}

/// Retrieve [Drivers] struct without check whether it has already been taken
/// at a different point.
/// # Safety
/// This shall only used in special situations where drivers cannot be passed as arguments
/// as in the panic handler. Otherwise global mutable state (as shared buffers) may be exposed
/// in an unsafe manner.
pub unsafe fn retrieve_drivers_unsafe() -> Drivers {
    DRIVERS
}

#[allow(clippy::declare_interior_mutable_const)]
const DRIVERS: Drivers = Drivers {
    adc: AdcDriverFactory,
    ble_advertising: BleAdvertisingDriverFactory,
    ble_scanning: BleScanningDriverFactory,
    buttons: ButtonsDriverFactory,
    console: ConsoleDriver,
    ctap: CtapDriverFactory,
    leds: LedsDriverFactory,
    timer: DriverContext {
        active_timer: Cell::new(None),
    },
    gpio: GpioDriverFactory,
    hmac: HmacDriverFactory,
    i2c_ms: I2cMSDriverFactory,
    temperature: TemperatureDriverFactory,
    rng: RngDriver,
    ambient_light_sensor: AmbientLightSensor,
    temperature_sensor: TemperatureSensor,
    humidity_sensor: HumiditySensor,
    ninedof: NinedofDriver,
};

pub struct DriversAlreadyTakenError;

impl From<DriversAlreadyTakenError> for TockError {
    fn from(_: DriversAlreadyTakenError) -> Self {
        TockError::Other(OtherError::DriversAlreadyTaken)
    }
}
