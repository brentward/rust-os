#[no_mangle]
#[cfg(not(test))]
#[lang = "panic_fmt"]
pub extern fn panic_fmt(fmt: ::std::fmt::Arguments, file: &'static str, line: u32, col: u32) -> ! {
    use console::kprintln;

    kprintln!("          ________");
    kprintln!("      (( /========\\                                                    _ ._  _ , _ ._");
    kprintln!("      __/__________\\____________n_                                   (_ ' ( `  )_  .__)");
    kprintln!("  (( /              \\_____________]                                ( (  (    )   `)  ) _)");
    kprintln!("    /  =(*)=          \\                                           (__ (_   (_ . _) _) ,__)");
    kprintln!("    |_._._._._._._._._.|         !                                    `~~`\\ ' . /`~~`");
    kprintln!("(( / __________________ \\       =o                                         ;   ;");
    kprintln!("  | OOOOOOOOOOOOOOOOOOO0 |   =                                             /   \\");
    kprintln!("__________________________________________________________________________/_ __ \\_____________");
    kprintln!("-------------------------------------------- PANIC --------------------------------------------");
    kprintln!("");
    kprintln!("FILE: {}", file);
    kprintln!("LINE: {}", line);
    kprintln!("COL: {}", col);
    kprintln!("");
    kprintln!("{}", fmt);

    loop { unsafe { asm!("wfe") } }
}

#[cfg(not(test))] #[lang = "eh_personality"] pub extern fn eh_personality() {}
