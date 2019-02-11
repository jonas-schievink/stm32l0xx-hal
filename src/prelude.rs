pub use embedded_hal::prelude::*;

pub use crate::gpio::GpioExt as _stm32f0xx_hal_gpio_GpioExt;
pub use crate::rcc::RccExt as _stm32f4xx_hal_rcc_RccExt;
pub use crate::time::MonoTimerExt as _stm32f4xx_hal_time_MonoTimerExt;
pub use crate::time::U32Ext as _stm32f4xx_hal_time_U32Ext;