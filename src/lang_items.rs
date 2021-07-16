use core::{alloc::Layout, panic::PanicInfo};

use crate::{log, log::eprintln};

extern "C" {
    fn drone_self_reset() -> !;
}

#[panic_handler]
fn begin_panic(pi: &PanicInfo<'_>) -> ! {
    eprintln!("{}", pi);
    abort()
}

#[lang = "oom"]
fn oom(layout: Layout) -> ! {
    eprintln!("Couldn't allocate memory of size {}. Aborting!", layout.size());
    abort()
}

fn abort() -> ! {
    log::flush();
    unsafe { drone_self_reset() }
}
