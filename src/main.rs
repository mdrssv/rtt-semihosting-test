#![no_std]
#![no_main]

extern crate cortex_m;

use cortex_m::{Peripherals, delay::Delay};
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{ChannelMode, DownChannel, rprint, rprintln, rtt_init, set_print_channel};
use semihosting::println as sprintln;
use arrayvec::ArrayString;
use core::fmt::Write as _;

#[entry]
unsafe fn main() -> ! {
    let cp = unsafe { Peripherals::steal() };
    let mut delay = Delay::new(cp.SYST, 64_000_000);
    let rtt = cfg!(feature = "rtt");
    let semihosting = cfg!(feature = "semihosting");
    let mut input: Option<DownChannel> = if rtt {
        #[cfg(feature = "rtt-cb")]
        let channels = {
            #[cfg(feature = "rtt-input")]
            let channels = rtt_init! {
                up: {
                    0: {
                        size: 1024,
                        mode: ChannelMode::NoBlockSkip,
                        name: "Terminal"
                    }
                }
                down: {
                    0: {
                        size: 16,
                        name: "Terminal"
                    }
                }
            };
            #[cfg(not(feature = "rtt-input"))]
            let channels = rtt_init! {
                up: {
                    0: {
                        size: 1024,
                        mode: ChannelMode::NoBlockSkip,
                        name: "Terminal"
                    }
                }
            };
            channels
        };
        #[cfg(feature = "rtt-cb")]
        set_print_channel(channels.up.0);
        #[allow(unused)]
        let down: Option<DownChannel> = None;
        #[cfg(all(feature = "rtt-input", feature = "rtt-cb"))]
        let down = Some(channels.down.0);
        down
    } else {
        None
    };
    let mut i = 0u32;
    let mut buf = ArrayString::<32>::new();
    loop {
        if rtt {
            rprintln!("Hello from RTT: {}", i);
        }
        if semihosting {
            buf.clear();
            // buffering reduces the number of semihosting syscalls
            let _ = write!(&mut buf,"Hello from semihosting: {}", i);
            sprintln!("{}", buf.as_str());
        }

        if i < 20 {
            delay.delay_ms(500);
        } else {
            delay.delay_ms(1_000 * 2);
        }
        i += 1;
        if let Some(ref mut input) = input {
            let mut buf = [0u8; 128];
            let read = input.read(&mut buf);
            if read == 0 {
                rprint!("o\r");
                continue;
            }
            if let Ok(s) = str::from_utf8(&buf[..read]) {
                if let Ok(n) = s.trim().parse() {
                    i = n;
                    rprintln!("restarting counter with: {}", n);
                }
                for line in s.lines() {
                    rprintln!("echo: {}", line);
                }
            } else {
                rprintln!("invalid string");
            }
        }
    }
}
