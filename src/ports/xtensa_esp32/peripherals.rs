use esp_hal::{peripherals::*, prelude::*};

pub static mut PERIPHERALS_VARIABLE: Option<Peripherals> = None;
pub struct Peripherals {
    pub i2c0: Option<I2C0>,
    pub spi2: Option<SPI2>,
    pub uart0: Option<UART0>,
    pub uart1: Option<UART1>,
    pub adc1: Option<ADC1>,
    pub bt: Option<BT>,
    pub dma: Option<DMA>,
    pub efuse: Option<EFUSE>,
    pub gpio: Option<GPIO>,
    pub io_mux: Option<IO_MUX>,
    pub ledc: Option<LEDC>,
    pub lpwr: Option<LPWR>,
    pub radio_clk: Option<RADIO_CLK>,
    pub rng: Option<RNG>,
    pub sha: Option<SHA>,
    pub spi0: Option<SPI0>,
    pub spi1: Option<SPI1>,
    pub system: Option<SYSTEM>,
    pub sw_interrupt: Option<SW_INTERRUPT>,
    pub wifi: Option<WIFI>,
}

impl Peripherals {
    pub fn new() -> Self {
        let peripherals = esp_hal::init(esp_hal::Config::default());

        Peripherals {
            i2c0: Some(peripherals.I2C0),
            spi2: Some(peripherals.SPI2),
            uart0: Some(peripherals.UART0),
            uart1: Some(peripherals.UART1),
            adc1: Some(peripherals.ADC1),
            bt: Some(peripherals.BT),
            dma: Some(peripherals.DMA),
            efuse: Some(peripherals.EFUSE),
            gpio: Some(peripherals.GPIO),
            io_mux: Some(peripherals.IO_MUX),
            ledc: Some(peripherals.LEDC),
            lpwr: Some(peripherals.LPWR),
            radio_clk: Some(peripherals.RADIO_CLK),
            rng: Some(peripherals.RNG),
            sha: Some(peripherals.SHA),
            spi0: Some(peripherals.SPI0),
            spi1: Some(peripherals.SPI1),
            system: Some(peripherals.SYSTEM),
            sw_interrupt: Some(peripherals.SW_INTERRUPT),
            wifi: Some(peripherals.WIFI),
        }
    }
}

pub fn init_peripherals() {
    unsafe { PERIPHERALS_VARIABLE = Some(Peripherals::new()) }
}
