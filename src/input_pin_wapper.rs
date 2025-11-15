use std::fmt::Debug;

use embedded_hal::digital::{Error, ErrorKind, ErrorType, InputPin};
use rppal::gpio;

#[derive(Debug, Clone, Copy)]
pub enum InputPinWapperError {
    Ok = 0,
}

impl Error for InputPinWapperError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Ok => ErrorKind::Other,
        }
    }
}

impl std::fmt::Display for InputPinWapperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for InputPinWapperError {}

/// Input Pin Wapper
pub struct InputPinWapper {
    pin: gpio::InputPin,
}

impl ErrorType for InputPinWapper {
    type Error = InputPinWapperError;
}

impl InputPinWapper {
    pub fn new(pin: gpio::InputPin) -> Self {
        Self { pin }
    }
}

impl InputPin for InputPinWapper {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(self.pin.is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.pin.is_low())
    }
}
