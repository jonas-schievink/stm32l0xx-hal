//! Encryption/decryption using the AES peripheral


#![no_main]
#![no_std]


extern crate panic_semihosting;


use core::pin::Pin;

use cortex_m::{
    asm,
    interrupt,
};
use cortex_m_rt::entry;
use stm32l0xx_hal::{
    prelude::*,
    aes::AES,
    dma::{
        self,
        DMA,
    },
    pac::{
        self,
        Interrupt,
    },
    rcc::Config,
};


#[entry]
fn main() -> ! {
    let mut cp = pac::CorePeripherals::take().unwrap();
    let     dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.freeze(Config::hsi16());
    let mut aes = AES::new(dp.AES, &mut rcc);
    let mut dma = DMA::new(dp.DMA1, &mut rcc);

    let key = [0x01234567, 0x89abcdef, 0x01234567, 0x89abcdef];
    let ivr = [0xfedcba98, 0x76543210, 0xfedcba98];

    const DATA: [u32; 8] = [
        0x00112233,
        0x44556677,
        0x8899aabb,
        0xccddeeff,

        0x00112233,
        0x44556677,
        0x8899aabb,
        0xccddeeff,
    ];
    let data = Pin::new(&DATA);

    static mut ENCRYPTED: [u32; 8] = [0; 8];
    static mut DECRYPTED: [u32; 8] = [0; 8];

    // Prepare DMA buffers. This is safe, as this is the `main` function, and no
    // other functions have access to these statics.
    let mut encrypted = Pin::new(unsafe { &mut ENCRYPTED });
    let mut decrypted = Pin::new(unsafe { &mut DECRYPTED });

    loop {
        let mut ctr_stream = aes.start_ctr_stream(key, ivr);
        let mut tx_transfer = ctr_stream.tx
            .write_all(
                &mut dma.handle,
                data,
                dma.channels.channel1,
            );
        let mut rx_transfer = ctr_stream.rx
            .read_all(
                &mut dma.handle,
                encrypted,
                dma.channels.channel2,
            );

        let (tx_res, rx_res) = interrupt::free(|_| {
            cp.NVIC.enable(Interrupt::DMA1_CHANNEL1);
            cp.NVIC.enable(Interrupt::DMA1_CHANNEL2_3);

            let interrupts = dma::Interrupts {
                transfer_error: true,
                transfer_complete: true,
                .. Default::default()
            };

            tx_transfer.enable_interrupts(interrupts);
            rx_transfer.enable_interrupts(interrupts);

            let tx_transfer = tx_transfer.start();
            let rx_transfer = rx_transfer.start();

            asm::wfi();

            let tx_res = tx_transfer.wait().unwrap();
            let rx_res = rx_transfer.wait().unwrap();

            cp.NVIC.disable(Interrupt::DMA1_CHANNEL1);
            cp.NVIC.disable(Interrupt::DMA1_CHANNEL2_3);

            (tx_res, rx_res)
        });

        ctr_stream.tx         = tx_res.target;
        ctr_stream.rx         = rx_res.target;
        dma.channels.channel1 = tx_res.channel;
        dma.channels.channel2 = rx_res.channel;
        encrypted             = rx_res.buffer;
        aes                   = ctr_stream.finish();

        assert_ne!(encrypted, Pin::new(&mut [0; 8]));
        assert_ne!(encrypted, data);

        let mut ctr_stream = aes.start_ctr_stream(key, ivr);
        let mut tx_transfer = ctr_stream.tx
            .write_all(
                &mut dma.handle,
                encrypted,
                dma.channels.channel1,
            );
        let mut rx_transfer = ctr_stream.rx
            .read_all(
                &mut dma.handle,
                decrypted,
                dma.channels.channel2,
            );

        let (tx_res, rx_res) = interrupt::free(|_| {
            cp.NVIC.enable(Interrupt::DMA1_CHANNEL1);
            cp.NVIC.enable(Interrupt::DMA1_CHANNEL2_3);

            let interrupts = dma::Interrupts {
                transfer_error: true,
                transfer_complete: true,
                .. Default::default()
            };

            tx_transfer.enable_interrupts(interrupts);
            rx_transfer.enable_interrupts(interrupts);

            let tx_transfer = tx_transfer.start();
            let rx_transfer = rx_transfer.start();

            asm::wfi();

            let tx_res = tx_transfer.wait().unwrap();
            let rx_res = rx_transfer.wait().unwrap();

            cp.NVIC.disable(Interrupt::DMA1_CHANNEL1);
            cp.NVIC.disable(Interrupt::DMA1_CHANNEL2_3);

            (tx_res, rx_res)
        });

        ctr_stream.tx         = tx_res.target;
        ctr_stream.rx         = rx_res.target;
        dma.channels.channel1 = tx_res.channel;
        dma.channels.channel2 = rx_res.channel;
        encrypted             = tx_res.buffer;
        decrypted             = rx_res.buffer;
        aes                   = ctr_stream.finish();

        assert_eq!(decrypted, data);
    }
}
