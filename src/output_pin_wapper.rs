use std::fmt::Debug;

use embedded_hal::digital::{Error, ErrorKind, ErrorType, OutputPin, PinState};
use rppal::gpio;

#[derive(Debug, Clone, Copy)]
pub enum OutputPinWapperError {
    Ok = 0,
}

impl Error for OutputPinWapperError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Ok => ErrorKind::Other,
        }
    }
}

impl std::fmt::Display for OutputPinWapperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for OutputPinWapperError {}

/// I/O Pin Wapper
pub struct OutputPinWapper {
    pin: gpio::OutputPin,
}

impl ErrorType for OutputPinWapper {
    type Error = OutputPinWapperError;
}

impl OutputPinWapper {
    pub fn new(pin: gpio::OutputPin) -> Self {
        Self { pin }
    }
}

impl OutputPin for OutputPinWapper {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.pin.set_high();
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.pin.set_low();
        Ok(())
    }

    fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> {
        match state {
            PinState::High => self.pin.set_high(),
            PinState::Low => self.pin.set_low(),
        }
        Ok(())
    }
}
