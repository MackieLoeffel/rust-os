use cga_screen::{Color, CGAScreen, ROWS, COLUMNS};

pub fn windows() {
    let mut screen = CGAScreen::new(0, 0, COLUMNS, ROWS);

    screen.set_color(Color::White, Color::Blue);
    screen.clear();
    println!(screen, "A problem has been detected and Windows has been shut down to prevent damage");
    println!(screen, "to your computer.");
    println!(screen, "");
    println!(screen, "The problem seems to be caused by the following file: SPCMDCON.SYS");
    println!(screen, "PAGE_FAULT_IN_NONPAGED_AREA");
    println!(screen, "If this is the first time you've seen this stop error screen,");
    println!(screen, "restart your computer. If this screen appears again, follow");
    println!(screen, "these steps:");
    println!(screen, "");
    println!(screen, "Check to make sure any new hardware or software is properly installed.");
    println!(screen, "If this is a new installation, ask your hardware or software manufacturer");
    println!(screen, "for any Windows updates you might need.");
    println!(screen, "");
    println!(screen, "If problems continue, disable or remove any newly installed hardware");
    println!(screen, "or software. Disable BIOS memory options such as caching or shadowing.");
    println!(screen, "If you need to use Safe Mode to remove or disable components, restart");
    println!(screen, "your computer, press F8 to select Advanced Startup Options, and then");
    println!(screen, "select Safe Mode.");
    println!(screen, "");
    println!(screen, "Technical information:");
    println!(screen, "");
    println!(screen, "*** STOP: 0x00000050 (0xFD3094C2,0x00000001,0xFBFE7617,0x00000000)");
    println!(screen, "");
    println!(screen, "***  SPCMDCON.SYS - Address FBFE7617 base at FBFE5000, DateStamp 3d6dd67c");

    loop {}
}
