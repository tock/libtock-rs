#![feature(alloc)]
#![no_std]

extern crate tock;

use core::fmt::Write;
use tock::console::Console;
use tock::sensors::*;
use tock::timer;
use tock::timer::Duration;

fn main() {
    let mut console = Console::new();
    let mut humidity = HumiditySensor;
    let mut temperature = TemperatureSensor;
    let mut light = AmbientLightSensor;
    let mut ninedof = unsafe { Ninedof::new() };
    loop {
        writeln!(console, "Humidity:    {}", humidity.read()).unwrap();
        writeln!(console, "Temperature: {}", temperature.read()).unwrap();
        writeln!(console, "Light:       {}", light.read()).unwrap();
        writeln!(console, "Accel:       {}", ninedof.read_acceleration()).unwrap();
        timer::sleep(Duration::from_ms(500));
    }
}
