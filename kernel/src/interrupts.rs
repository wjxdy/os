use core::fmt::Write;

use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::sync::SpinMutex;

static IDT: SpinMutex<InterruptDescriptorTable> = SpinMutex::new(InterruptDescriptorTable::new());

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    let args = format_args!("EXCEPTION: BREAKPOINT\n{stack_frame:#?}\n");

    if !crate::logger::try_print(args) {
        let mut serial = crate::serial_port();
        let _ = writeln!(serial, "EMERGENCY EXCEPTION: BREAKPOINT\n{stack_frame:#?}");
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let accessed_address = Cr2::read_raw();
    let args = format_args!(
        "EXCEPTION: PAGE FAULT\n\
        accessed address: {accessed_address:#x}\n\
        error code: {error_code:?}\n\
        {stack_frame:#?}\n"
    );

    if !crate::logger::try_print_panic(args) {
        let mut serial = crate::serial_port();
        let _ = writeln!(
            serial,
            "EXCEPTION: PAGE FAULT\n\
            accessed address: {accessed_address:#x}\n\
            error code: {error_code:?}\n\
            {stack_frame:#?}\n"
        );
    }

    halt_loop();
}

fn halt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init() {
    let mut idt = IDT.lock();

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);

    unsafe {
        idt.load_unsafe();
    }
}
