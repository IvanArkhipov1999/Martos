use esp_hal::{peripherals::*, prelude::*};

pub static mut PERIPHERALS_VARIABLE: Option<Peripherals> = None;
pub struct Peripherals {
    pub i2c0: Option<I2C0>,
    pub spi2: Option<SPI2>,
    pub uart0: Option<UART0>,
    pub uart1: Option<UART1>,
    pub adc1: Option<ADC1>,
    pub apb_ctrl: Option<APB_CTRL>,
    pub apb_saradc: Option<APB_SARADC>,
    pub assist_debug: Option<ASSIST_DEBUG>,
    pub bb: Option<BB>,
    pub bt: Option<BT>,
    pub dma: Option<DMA>,
    pub ecc: Option<ECC>,
    pub efuse: Option<EFUSE>,
    pub extmem: Option<EXTMEM>,
    pub gpio: Option<GPIO>,
    pub i2c_ana_mst: Option<I2C_ANA_MST>,
    pub interrupt_core0: Option<INTERRUPT_CORE0>,
    pub io_mux: Option<IO_MUX>,
    pub ledc: Option<LEDC>,
    pub lpwr: Option<LPWR>,
    pub modem_clkrst: Option<MODEM_CLKRST>,
    pub radio_clk: Option<RADIO_CLK>,
    pub rng: Option<RNG>,
    pub sensitive: Option<SENSITIVE>,
    pub sha: Option<SHA>,
    pub spi0: Option<SPI0>,
    pub spi1: Option<SPI1>,
    pub system: Option<SYSTEM>,
    pub systimer: Option<SYSTIMER>,
    pub sw_interrupt: Option<SW_INTERRUPT>,
    pub timg0: Option<TIMG0>,
    pub wifi: Option<WIFI>,
    pub xts_aes: Option<XTS_AES>,
    pub mem2mem1: Option<MEM2MEM1>,
    pub mem2mem2: Option<MEM2MEM2>,
    pub mem2mem3: Option<MEM2MEM3>,
    pub mem2mem4: Option<MEM2MEM4>,
    pub mem2mem5: Option<MEM2MEM5>,
    pub mem2mem6: Option<MEM2MEM6>,
    pub mem2mem8: Option<MEM2MEM8>,
    pub dma_ch0: Option<DmaChannel0>,
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
            apb_ctrl: Some(peripherals.APB_CTRL),
            apb_saradc: Some(peripherals.APB_SARADC),
            assist_debug: Some(peripherals.ASSIST_DEBUG),
            bb: Some(peripherals.BB),
            bt: Some(peripherals.BT),
            dma: Some(peripherals.DMA),
            ecc: Some(peripherals.ECC),
            efuse: Some(peripherals.EFUSE),
            extmem: Some(peripherals.EXTMEM),
            gpio: Some(peripherals.GPIO),
            i2c_ana_mst: Some(peripherals.I2C_ANA_MST),
            interrupt_core0: Some(peripherals.INTERRUPT_CORE0),
            io_mux: Some(peripherals.IO_MUX),
            ledc: Some(peripherals.LEDC),
            lpwr: Some(peripherals.LPWR),
            modem_clkrst: Some(peripherals.MODEM_CLKRST),
            radio_clk: Some(peripherals.RADIO_CLK),
            rng: Some(peripherals.RNG),
            sensitive: Some(peripherals.SENSITIVE),
            sha: Some(peripherals.SHA),
            spi0: Some(peripherals.SPI0),
            spi1: Some(peripherals.SPI1),
            system: Some(peripherals.SYSTEM),
            systimer: Some(peripherals.SYSTIMER),
            sw_interrupt: Some(peripherals.SW_INTERRUPT),
            timg0: Some(peripherals.TIMG0),
            wifi: Some(peripherals.WIFI),
            xts_aes: Some(peripherals.XTS_AES),
            mem2mem1: Some(peripherals.MEM2MEM1),
            mem2mem2: Some(peripherals.MEM2MEM2),
            mem2mem3: Some(peripherals.MEM2MEM3),
            mem2mem4: Some(peripherals.MEM2MEM4),
            mem2mem5: Some(peripherals.MEM2MEM5),
            mem2mem6: Some(peripherals.MEM2MEM6),
            mem2mem8: Some(peripherals.MEM2MEM8),
            dma_ch0: Some(peripherals.DmaChannel0),
        }
    }
}

pub fn init_peripherals() {
    unsafe {
        PERIPHERALS_VARIABLE = Some(Peripherals::new())
    }
}
