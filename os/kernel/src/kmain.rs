// #![feature(compiler_builtins_lib, lang_items, asm, pointer_methods)]
// #![no_builtins]
// #![no_std]
//
// extern crate compiler_builtins;
// extern crate pi;
//
// use pi::timer::spin_sleep_ms;
// use pi::gpio::Gpio;
//
// pub mod lang_items;
//
//
// // const GPIO_BASE: usize = 0x3F000000 + 0x200000;
// //
// // const GPIO_FSEL1: *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
// // const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
// // const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;
//
// #[no_mangle]
// pub extern "C" fn kmain() {
//     let mut interval = 150i64;
//     let max_interval = interval.clone();
//     let pins = [16u8, 5, 6, 13, 19, 26];
//
//
//     // let mut pin_16 = Gpio::new(16).into_output();
//     // let mut pin_5 = Gpio::new(5).into_output();
//     // let mut pin_6 = Gpio::new(6).into_output();
//     // let mut pin_13 = Gpio::new(13).into_output();
//     // let mut pin_19 = Gpio::new(19).into_output();
//     // let mut pin_26 = Gpio::new(26).into_output();
//     loop {
//         while interval > 40 {
//             for pin in pins.iter() {
//                 Gpio::new(*pin).into_output().set();
//                 spin_sleep_ms(interval as u64);
//                 Gpio::new(*pin).into_output().clear();
//             }
//             interval -= interval / 20;
//             // pin_16.set();
//             // spin_sleep_ms(interval);
//             // pin_16.clear();
//             // pin_5.set();
//             // spin_sleep_ms(interval);
//             // pin_5.clear();
//             // pin_6.set();
//             // spin_sleep_ms(interval);
//             // pin_6.clear();
//             // pin_13.set();
//             // spin_sleep_ms(interval);
//             // pin_13.clear();
//             // pin_19.set();
//             // spin_sleep_ms(interval);
//             // pin_19.clear();
//             // pin_26.set();
//             // spin_sleep_ms(interval);
//             // pin_26.clear();
//         }
//         while interval <= max_interval {
//             for pin in pins.iter() {
//                 Gpio::new(*pin).into_output().set();
//                 spin_sleep_ms(interval as u64);
//                 Gpio::new(*pin).into_output().clear();
//             }
//             interval += interval / 19;
//         }
//     }
// }
#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(never_type)]
#![feature(ptr_internals)]

extern crate pi;
extern crate stack_vec;

use pi::uart::MiniUart;
use std::io::{Read, Write};
use std::fmt::Write as OtherWrite;

pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;

#[no_mangle]
pub extern "C" fn kmain() {
    pi::timer::spin_sleep_ms(5000);
    let mut uart = MiniUart::new();
    uart.write_str("This is the beginning\nand another line\nand another\n");

    loop {
        // let byte = b'b';
        let byte = uart.read_byte();
        uart.write_str("hello foo\n");
        uart.write_byte(byte);
        // <MiniUart as OtherWrite>::write_str(&mut uart, "<-");
        uart.write_str("<-\n").unwrap();
    }

    // let mut interval = 150i64;
    // let max_interval = interval.clone();
    // let pins = [16u8, 5, 6, 13, 19, 26];
    // loop {
    //     while interval > 40 {
    //         for pin in pins.iter() {
    //             Gpio::new(*pin).into_output().set();
    //             spin_sleep_ms(interval as u64);
    //             Gpio::new(*pin).into_output().clear();
    //         }
    //         interval -= interval / 20;
    //     }
    //     while interval <= max_interval {
    //         for pin in pins.iter() {
    //             Gpio::new(*pin).into_output().set();
    //             spin_sleep_ms(interval as u64);
    //             Gpio::new(*pin).into_output().clear();
    //         }
    //         interval += interval / 19;
    //     }
    // }
}
