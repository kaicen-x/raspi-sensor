use std::fmt::Debug;

use embedded_hal::digital::{Error, ErrorKind, ErrorType, InputPin, OutputPin, PinState};
use rppal::gpio::{IoPin, Mode};

#[derive(Debug, Clone, Copy)]
pub enum IoPinWapperError {
    Ok = 0,
}

impl Error for IoPinWapperError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Ok => ErrorKind::Other,
        }
    }
}

impl std::fmt::Display for IoPinWapperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for IoPinWapperError {}

/// I/O Pin Wapper
pub struct IoPinWapper {
    pin: IoPin,
    mode: Mode,
}

impl ErrorType for IoPinWapper {
    type Error = IoPinWapperError;
}

impl IoPinWapper {
    pub fn new(pin: IoPin) -> Self {
        Self {
            pin,
            mode: Mode::Null,
        }
    }
}

impl InputPin for IoPinWapper {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        if self.mode != Mode::Input {
            self.pin.set_mode(Mode::Input);
            self.mode = Mode::Input;
        }

        Ok(self.pin.is_high())
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        if self.mode != Mode::Input {
            self.pin.set_mode(Mode::Input);
            self.mode = Mode::Input;
        }

        Ok(self.pin.is_low())
    }
}

impl OutputPin for IoPinWapper {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        if self.mode != Mode::Output {
            self.pin.set_mode(Mode::Output);
            self.mode = Mode::Output;
        }

        self.pin.set_high();
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        if self.mode != Mode::Output {
            self.pin.set_mode(Mode::Output);
            self.mode = Mode::Output;
        }

        self.pin.set_low();
        Ok(())
    }

    fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> {
        if self.mode != Mode::Output {
            self.pin.set_mode(Mode::Output);
            self.mode = Mode::Output;
        }

        match state {
            PinState::High => self.pin.set_high(),
            PinState::Low => self.pin.set_low(),
        }
        Ok(())
    }
}
