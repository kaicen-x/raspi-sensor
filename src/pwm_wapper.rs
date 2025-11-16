use embedded_hal_0::PwmPin;
use std::fmt::Debug;

#[derive(Debug)]
pub enum PwmWapperError {
    Pwm(rppal::pwm::Error),
}

impl embedded_hal::pwm::Error for PwmWapperError {
    fn kind(&self) -> embedded_hal::pwm::ErrorKind {
        embedded_hal::pwm::ErrorKind::Other
    }
}

impl std::fmt::Display for PwmWapperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl std::error::Error for PwmWapperError {}

/// PWM Wapper
pub struct PwmWapper {
    pin: rppal::pwm::Pwm,
}

impl embedded_hal::pwm::ErrorType for PwmWapper {
    type Error = PwmWapperError;
}

impl PwmWapper {
    pub fn new(pin: rppal::pwm::Pwm) -> Self {
        Self { pin }
    }
}

impl embedded_hal::pwm::SetDutyCycle for PwmWapper {
    fn max_duty_cycle(&self) -> u16 {
        self.pin.get_max_duty() as u16
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        self.pin
            .set_duty_cycle(duty as f64 / self.pin.get_max_duty())
            .map_err(|err| PwmWapperError::Pwm(err))?;
        Ok(())
    }
}
