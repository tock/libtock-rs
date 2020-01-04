use crate::adc::AdcDriver;
use crate::buttons::ButtonDriver;
use crate::console::ConsoleDriver;
use crate::gpio::GpioDriver;
use crate::led::LedDriverFactory;
use crate::result::OtherError;
use crate::result::TockError;
use crate::result::TockResult;
use crate::rng::RngDriver;
use crate::sensors::ninedof::NinedofDriver;
use crate::sensors::AmbientLightSensor;
use crate::sensors::HumiditySensor;
use crate::sensors::TemperatureSensor;
use crate::simple_ble::BleAdvertisingDriver;
use crate::simple_ble::BleScanningDriver;
use crate::temperature::TemperatureDriver;
use crate::timer::DriverContext;
use core::cell::Cell;

/// Struct containing all drivers constructible through [retrieve_drivers()]
#[non_exhaustive]
pub struct Drivers {
    pub console_driver: ConsoleDriver,
    pub led_driver_factory: LedDriverFactory,
    pub timer_context: DriverContext,
    pub gpio_driver: GpioDriver,
    pub temperature_driver: TemperatureDriver,
    pub button_driver: ButtonDriver,
    pub adc_driver: AdcDriver,
    pub rng_driver: RngDriver,
    pub ble_advertising_driver: BleAdvertisingDriver,
    pub ble_scanning_driver: BleScanningDriver,
    pub ambient_light_sensor: AmbientLightSensor,
    pub temperature_sensor: TemperatureSensor,
    pub humidity_sensor: HumiditySensor,
    pub ninedof_driver: NinedofDriver,
}

/// Retrieve [Drivers] struct. Returns struct only once.
pub fn retrieve_drivers() -> TockResult<Drivers> {
    match unsafe { DRIVERS_SINGLETON.take() } {
        Some(drivers) => Ok(drivers),
        None => Err(TockError::Other(OtherError::DriverAlreadyTaken)),
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
    adc_driver: AdcDriver,
    ble_advertising_driver: BleAdvertisingDriver,
    ble_scanning_driver: BleScanningDriver,
    button_driver: ButtonDriver,
    console_driver: ConsoleDriver,
    led_driver_factory: LedDriverFactory,
    timer_context: DriverContext {
        active_timer: Cell::new(None),
    },
    gpio_driver: GpioDriver,
    temperature_driver: TemperatureDriver,
    rng_driver: RngDriver,
    ambient_light_sensor: AmbientLightSensor,
    temperature_sensor: TemperatureSensor,
    humidity_sensor: HumiditySensor,
    ninedof_driver: NinedofDriver,
};

static mut DRIVERS_SINGLETON: Option<Drivers> = Some(DRIVERS);

#[cfg(test)]
mod test {
    use super::DRIVERS;
    use super::DRIVERS_SINGLETON;
    use crate::retrieve_drivers;
    #[test]
    pub fn can_be_retrieved_once() {
        reset_drivers_singleton();

        assert!(retrieve_drivers().is_ok());
        assert!(retrieve_drivers().is_err());
    }

    fn reset_drivers_singleton() {
        unsafe {
            DRIVERS_SINGLETON = Some(DRIVERS);
        };
    }
}
