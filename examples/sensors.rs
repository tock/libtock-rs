#![no_std]

use core::fmt::Write;
use libtock::console::Console;
use libtock::sensors::*;
use libtock::timer;
use libtock::timer::Duration;

fn main() {
    let mut console = Console::new();
    let mut humidity = HumiditySensor;
    let mut temperature = TemperatureSensor;
    let mut light = AmbientLightSensor;
    let mut ninedof = unsafe { Ninedof::new() };
    loop {
        writeln!(console, "Humidity:    {}\n", humidity.read()).unwrap();
        writeln!(console, "Temperature: {}\n", temperature.read()).unwrap();
        writeln!(console, "Light:       {}\n", light.read()).unwrap();
        writeln!(console, "Accel:       {}\n", ninedof.read_acceleration()).unwrap();
        timer::sleep(Duration::from_ms(500));
    }
}
