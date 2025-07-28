use crate::{command_return, DriverInfo, DriverShareRef, RoAllowBuffer};
use core::cell::Cell;
use libtock_platform::{CommandReturn, ErrorCode};

pub struct Screen {
    screen_setup: Option<u16>,     // Optional screen setup state
    resolution_modes: Option<u16>, // Number of supported resolution modes
    invert: Cell<bool>,            // Current invert state (true = inverted)
    screen_resolution_width_height: [Option<(u16, u16)>; 3], // Predefined resolutions
    resolution_width_height: [Cell<u16>; 2], // Current resolution (width, height)
    pixel_modes: Option<u16>,      // Number of pixel formats supported
    screen_pixel_format: [u16; 2], // Predefined pixel formats
    pixel_format: Cell<u32>,       // Currently selected pixel format
    brightness: Cell<u16>,         // Current brightness level
    rotation: Cell<u16>,           // Current screen rotation
    write_frame: [Cell<u16>; 2],   // Coordinates for the write frame
    power: Cell<u16>,              // Power state
    share_ref: DriverShareRef,     // Handle for kernel-user communication
    write_buffer: Cell<Option<RoAllowBuffer>>, // Optional buffer for write operations
    messages: Cell<Vec<u8>>,
}

impl Screen {
    pub fn new() -> std::rc::Rc<Screen> {
        #[allow(clippy::declare_interior_mutable_const)]
        const VALUE_U16: Cell<u16> = Cell::new(0);
        #[allow(clippy::declare_interior_mutable_const)]
        const VALUE_U32: Cell<u32> = Cell::new(0);
        std::rc::Rc::new(Screen {
            screen_setup: Some(3),
            screen_pixel_format: [332, 565], // Example pixel formats
            screen_resolution_width_height: [
                Some((1920, 1080)),
                Some((2560, 1440)),
                Some((1280, 720)),
            ],
            pixel_format: VALUE_U32,
            resolution_modes: Some(2),
            resolution_width_height: [VALUE_U16, VALUE_U16],
            invert: Cell::new(false),
            brightness: VALUE_U16,
            pixel_modes: Some(5),
            rotation: VALUE_U16,
            write_frame: [VALUE_U16, VALUE_U16],
            power: VALUE_U16,
            share_ref: Default::default(),
            write_buffer: Cell::new(None),
            messages: Default::default(),
        })
    }

    pub fn take_bytes(&self) -> Vec<u8> {
        self.messages.take()
    }

    // Checks if the buffer size is compatible with the pixel format
    fn is_buffer_length_valid(&self, buffer_len: usize) -> bool {
        let bytes_per_pixel = match self.pixel_format.get() {
            1 => 1,            // Monochrome
            2 => 2,            // RGB_565
            3 => 3,            // RGB_888
            4 => 4,            // ARGB_8888
            _ => return false, // Unknown/unsupported format
        };

        buffer_len % bytes_per_pixel == 0
    }

    // Simulates writing to the screen
    fn write(&self, buffer: &[u8]) -> Result<(), ErrorCode> {
        if !self.is_buffer_length_valid(buffer.len()) {
            return Err(ErrorCode::Invalid);
        }

        self.share_ref
            .schedule_upcall(0, (0, 0, 0))
            .expect("Unable to schedule upcall");
        Ok(())
    }

    // Simulates filling the screen with a color
    fn fill(&self, _color: u16) -> Result<(), ErrorCode> {
        self.share_ref
            .schedule_upcall(0, (0, 0, 0))
            .expect("Unable to schedule upcall");
        Ok(())
    }
}

impl crate::fake::SyscallDriver for Screen {
    fn info(&self) -> DriverInfo {
        DriverInfo::new(DRIVER_NUM).upcall_count(3)
    }

    fn register(&self, share_ref: DriverShareRef) {
        self.share_ref.replace(share_ref);
    }

    fn allow_readonly(
        &self,
        buffer_num: u32,
        buffer: RoAllowBuffer,
    ) -> Result<RoAllowBuffer, (RoAllowBuffer, ErrorCode)> {
        if buffer_num == WRITE_BUFFER_ID {
            let old_buffer = self.write_buffer.replace(Some(buffer));
            Ok(old_buffer.unwrap_or_default())
        } else {
            Err((buffer, ErrorCode::Invalid))
        }
    }

    fn command(&self, command_num: u32, argument0: u32, argument1: u32) -> CommandReturn {
        match command_num {
            EXISTS => command_return::success(),

            SCREEN_SETUP => command_return::success_u32(self.screen_setup.unwrap() as u32),

            SET_POWER => {
                self.power.set(1);
                command_return::success()
            }

            GET_POWER => command_return::success_u32(self.power.get() as u32),

            SET_BRIGHTNESS => {
                self.brightness.set(argument0 as u16);
                self.share_ref
                    .schedule_upcall(0, (0, 0, 0))
                    .expect("Unable to schedule upcall {}");
                crate::command_return::success()
            }
            GET_BRIGHTNESS => crate::command_return::success_u32(self.brightness.get() as u32),

            SET_INVERT_ON => {
                self.share_ref
                    .schedule_upcall(0, (0, 0, 0))
                    .expect("Unable to schedule upcall {}");
                self.invert.set(argument0 != 0);
                crate::command_return::success()
            }

            SET_INVERT_OFF => {
                self.share_ref
                    .schedule_upcall(0, (0, 0, 0))
                    .expect("Unable to schedule upcall {}");
                self.invert.set(argument0 != 0);
                command_return::success()
            }

            SET_INVERT => {
                self.invert.set(argument0 != 0);
                command_return::success()
            }

            GET_INVERT => command_return::success_u32(self.invert.get() as u32),

            GET_RESOLUTION_MODES_COUNT => {
                if Option::is_some(&self.screen_setup) {
                    crate::command_return::success_u32(self.resolution_modes.unwrap() as u32)
                } else {
                    command_return::failure(ErrorCode::NoSupport)
                }
            }

            GET_RESOLUTION_WIDTH_HEIGHT => {
                self.share_ref
                    .schedule_upcall(0, (0, 0, 0))
                    .expect("Unable to schedule upcall {}");
                if argument0 < self.screen_resolution_width_height.len() as u32 {
                    if Option::is_some(&self.screen_setup) {
                        crate::command_return::success_2_u32(
                            self.screen_resolution_width_height[argument0 as usize]
                                .unwrap()
                                .0 as u32,
                            self.screen_resolution_width_height[argument0 as usize]
                                .unwrap()
                                .1 as u32,
                        )
                    } else {
                        command_return::failure(ErrorCode::NoSupport)
                    }
                } else {
                    command_return::failure(ErrorCode::Invalid)
                }
            }

            PIXEL_MODES_COUNT => {
                if self.screen_setup.is_some() {
                    command_return::success_u32(self.pixel_modes.unwrap() as u32)
                } else {
                    command_return::failure(ErrorCode::NoSupport)
                }
            }

            PIXEL_FORMAT => {
                self.share_ref
                    .schedule_upcall(0, (0, 0, 0))
                    .expect("Unable to schedule upcall {}");
                if argument0 < self.screen_pixel_format.len() as u32 {
                    if Option::is_some(&self.screen_setup) {
                        crate::command_return::success_u32(
                            self.screen_pixel_format[argument0 as usize] as u32,
                        )
                    } else {
                        crate::command_return::failure(ErrorCode::NoSupport)
                    }
                } else {
                    command_return::failure(ErrorCode::Invalid)
                }
            }

            GET_ROTATION => {
                self.share_ref
                    .schedule_upcall(0, (0, 0, 0))
                    .expect("Unable to schedule upcall {}");
                crate::command_return::success_u32(self.rotation.get() as u32)
            }

            SET_ROTATION => {
                self.share_ref
                    .schedule_upcall(0, (0, 0, 0))
                    .expect("Unable to schedule upcall {}");
                if argument0 > 359 {
                    command_return::failure(ErrorCode::Invalid)
                } else {
                    self.rotation.set(argument0 as u16);
                    command_return::success()
                }
            }

            GET_RESOLUTION => command_return::success_2_u32(
                self.resolution_width_height[0].get() as u32,
                self.resolution_width_height[1].get() as u32,
            ),

            SET_RESOLUTION => {
                self.share_ref
                    .schedule_upcall(0, (0, 0, 0))
                    .expect("Unable to schedule upcall {}");
                self.resolution_width_height[0].set(argument0 as u16);
                self.resolution_width_height[1].set(argument1 as u16);
                command_return::success()
            }

            GET_PIXEL_FORMAT => command_return::success_u32(self.pixel_format.get()),

            SET_PIXEL_FORMAT => {
                self.share_ref
                    .schedule_upcall(0, (0, 0, 0))
                    .expect("Unable to schedule upcall {}");
                if argument0 < self.pixel_modes.unwrap() as u32 {
                    self.pixel_format.set(argument0);
                    command_return::success()
                } else {
                    command_return::failure(ErrorCode::Invalid)
                }
            }

            SET_WRITE_FRAME => {
                self.share_ref
                    .schedule_upcall(0, (0, 0, 0))
                    .expect("Unable to schedule upcall {}");
                self.write_frame[0].set(argument0 as u16);
                self.write_frame[1].set(argument1 as u16);
                command_return::success()
            }

            GET_WRITE_FRAME => command_return::success_2_u32(
                self.write_frame[0].get() as u32,
                self.write_frame[1].get() as u32,
            ),

            WRITE => {
                let buffer_len = argument0 as usize;
                let buffer = self
                    .write_buffer
                    .take()
                    .expect("No buffer provided for WRITE command");

                if buffer.len() != buffer_len || !self.is_buffer_length_valid(buffer_len) {
                    return command_return::failure(ErrorCode::Invalid);
                }

                match self.write(buffer.as_ref()) {
                    Ok(()) => command_return::success(),
                    Err(e) => command_return::failure(e),
                }
            }

            FILL => {
                let color = argument0 as u16;
                match self.fill(color) {
                    Ok(()) => command_return::success(),
                    Err(e) => command_return::failure(e),
                }
            }

            _ => command_return::failure(ErrorCode::NoSupport),
        }
    }
}

// -----------------------------------------------------------------------------
// Implementation details below
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests;

const DRIVER_NUM: u32 = 0x90001;
const WRITE_BUFFER_ID: u32 = 0; // Buffer ID for write operations

// Command IDs
#[allow(unused)]
pub const EXISTS: u32 = 0;
pub const SCREEN_SETUP: u32 = 1;
pub const SET_POWER: u32 = 2;
pub const SET_BRIGHTNESS: u32 = 3;
pub const SET_INVERT_ON: u32 = 4;
pub const SET_INVERT_OFF: u32 = 5;
pub const SET_INVERT: u32 = 6;
pub const GET_RESOLUTION_MODES_COUNT: u32 = 11;
pub const GET_RESOLUTION_WIDTH_HEIGHT: u32 = 12;
pub const PIXEL_MODES_COUNT: u32 = 13;
pub const PIXEL_FORMAT: u32 = 14;
pub const GET_ROTATION: u32 = 21;
pub const SET_ROTATION: u32 = 22;
pub const GET_RESOLUTION: u32 = 23;
pub const SET_RESOLUTION: u32 = 24;
pub const GET_PIXEL_FORMAT: u32 = 25;
pub const SET_PIXEL_FORMAT: u32 = 26;
pub const SET_WRITE_FRAME: u32 = 100;
pub const WRITE: u32 = 200;
pub const FILL: u32 = 300;
pub const GET_POWER: u32 = 400;
pub const GET_BRIGHTNESS: u32 = 401;
pub const GET_INVERT: u32 = 402;
pub const GET_WRITE_FRAME: u32 = 403;
