/*
 * GNU linker script for Espressif ESP32
 */

/* Default entry point */
ENTRY( call_start_cpu0 );

/* Specify main memory areas */
MEMORY
{
  /* Use values from the ESP-IDF 'bootloader' component.
  /* TODO: Use human-readable lengths */
  /* TODO: Use the full memory map - this is just a test */
  iram_seg ( RX )       : ORIGIN = 0x40080400, len = 0xFC00
  dram_seg ( RW )       : ORIGIN = 0x3FFF0000, len = 0x8048
}

/* Define output sections */
SECTIONS {
  /* The program code and other data goes into Instruction RAM */
  .iram.text :
  {
    . = ALIGN(16);
    KEEP(*(.entry.text))
    *(.text)
    *(.text*)
    KEEP (*(.init))
    KEEP (*(.fini))
    *(.rodata)
    *(.rodata*)

    . = ALIGN(4);
    _etext = .;
  } >iram_seg

  /* Initialized data goes into Data RAM */
  _sidata = .;
  .data : AT(_sidata)
  {
    . = ALIGN(4);
    _sdata = .;
    *(.data)
    *(.data*)

    . = ALIGN(4);
    _edata = .;
  } >dram_seg

  /* Uninitialized data also goes into Data RAM */
  .bss :
  {
    . = ALIGN(4);
    _sbss = .;
    ExceptionHandler = .;
    _bss_start = .;
    _rtc_fast_bss_start = .;
    rtc_get_reset_reason = .;
    _rtc_fast_persistent_start = .;
    __global_pointer$ = .;
    _max_hart_id = .;
    _stack_start = .;
    interrupt20 = .;
    interrupt21 = .;
    interrupt22 = .;
    interrupt23 = .;
    interrupt24 = .;
    interrupt25 = .;
    interrupt26 = .;
    interrupt27 = .;
    interrupt28 = .;
    interrupt29 = .;
    interrupt30 = .;
    interrupt31 = .;
    ets_delay_us = .;
    ets_delay_us = .;
    ets_delay_us = .;
    ets_delay_us = .;
    WIFI_MAC = .;
    WIFI_MAC_NMI = .;
    WIFI_PWR = .;
    WIFI_BB = .;
    BT_MAC = .;
    BT_BB = .;
    BT_BB_NMI = .;
    LP_TIMER = .;
    COEX = .;
    BLE_TIMER = .;
    BLE_SEC = .;
    I2C_MASTER = .;
    ZB_MAC = .;
    PMU = .;
    EFUSE = .;
    LP_RTC_TIMER = .;
    LP_UART = .;
    LP_I2C = .;
    LP_WDT = .;
    LP_PERI_TIMEOUT = .;
    LP_APM_M0 = .;
    LP_APM_M1 = .;
    FROM_CPU_INTR0 = .;
    FROM_CPU_INTR1 = .;
    FROM_CPU_INTR2 = .;
    FROM_CPU_INTR3 = .;
    ASSIST_DEBUG = .;
    TRACE = .;
    CACHE = .;
    CPU_PERI_TIMEOUT = .;
    GPIO = .;
    GPIO_NMI = .;
    PAU = .;
    HP_PERI_TIMEOUT = .;
    MODEM_PERI_TIMEOUT = .;
    HP_APM_M0 = .;
    HP_APM_M1 = .;
    HP_APM_M2 = .;
    HP_APM_M3 = .;
    LP_APM0 = .;
    MSPI = .;
    I2S0 = .;
    UHCI0 = .;
    UART0 = .;
    UART1 = .;
    LEDC = .;
    TWAI0 = .;
    TWAI1 = .;
    USB_DEVICE = .;
    RMT = .;
    I2C_EXT0 = .;
    TG0_T0_LEVEL = .;
    TG0_T1_LEVEL = .;
    TG0_WDT_LEVEL = .;
    TG1_T0_LEVEL = .;
    TG1_T1_LEVEL = .;
    TG1_WDT_LEVEL = .;
    SYSTIMER_TARGET0 = .;
    SYSTIMER_TARGET1 = .;
    SYSTIMER_TARGET2 = .;
    APB_SARADC = .;
    MCPWM0 = .;
    PCNT = .;
    PARL_IO = .;
    SLC0 = .;
    SLC1 = .;
    DMA_IN_CH0 = .;
    DMA_IN_CH1 = .;
    DMA_IN_CH2 = .;
    DMA_OUT_CH0 = .;
    DMA_OUT_CH1 = .;
    DMA_OUT_CH2 = .;
    SPI2 = .;
    AES = .;
    SHA = .;
    RSA = .;
    ECC = .;
    *(.bss)
    *(.bss*)
    *(COMMON)

    . = ALIGN(4);
    _ebss = .;
    _bss_end = .;
    _rtc_fast_bss_end = .;
    _rtc_fast_persistent_end = .;
  } >dram_seg

  . = ALIGN(4);
  PROVIDE ( end = . );
  PROVIDE ( _end = . );
}
