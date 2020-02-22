#[no_mangle]
#[cfg(not(test))]
#[lang = "panic_fmt"]
pub extern fn panic_fmt(fmt: ::std::fmt::Arguments, file: &'static str, line: u32, col: u32) -> ! {
    use console::kprintln;

    kprintln!("                             ____");
    kprintln!("                     __,-~~/~    `---.");
    kprintln!("                   _/_,---(      ,    )");
    kprintln!("               __ /        <    /   )  \\___");
    kprintln!("- ------===;;;'====------------------===;;;===----- -  -");
    kprintln!("                  \\/  ~\"~\"~\"~\"~\"~\\~\"~)~\"/");
    kprintln!("                  (_ (   \\  (     >    \\)");
    kprintln!("                   \\_( _ <         >_>'");
    kprintln!("                      ~ `-i' ::>|--\"");
    kprintln!("                          I;|.|.|");
    kprintln!("                         <|i::|i|`.");
    kprintln!("                        (` ^'\"'-' \")");
    kprintln!("-----------------------------------------------------------");
    kprintln!("-------------------------- PANIC --------------------------");
    kprintln!("");
    kprintln!("FILE: {}", file);
    kprintln!("LINE: {}", line);
    kprintln!("COL: {}", col);
    kprintln!("");
    kprintln!("{}", fmt);

    loop { unsafe { asm!("wfe") } }
}

#[cfg(not(test))] #[lang = "eh_personality"] pub extern fn eh_personality() {}
