

#![allow(non_camel_case_types)]
pub type c_int = i32;
pub type c_uchar = u8;
pub type c_uint = u32;
pub type cc_t = c_uchar;
pub type speed_t = c_uint;
pub type tcflag_t = c_uint;

const STDIN_FILENO: i32 = 0;
pub const ICANON: tcflag_t = 0x00000100;
pub const ECHO: tcflag_t = 0o000010;
pub const TCSANOW: c_int = 0;
pub const NCCS: usize = 32;

static mut ORIGINAL_TIO: termios = termios {
    c_iflag: 0,
    c_oflag: 0,
    c_cflag: 0,
    c_lflag: 0,
    c_line: 0,
    c_cc: [0; 32],
    c_ispeed: 0,
    c_ospeed: 0,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct termios {
    pub c_iflag: tcflag_t,
    pub c_oflag: tcflag_t,
    pub c_cflag: tcflag_t,
    pub c_lflag: tcflag_t,
    c_line: cc_t,
    pub c_cc: [cc_t; NCCS],
    c_ispeed: speed_t,
    c_ospeed: speed_t,
}

pub fn disable_input_buffering() {
    unsafe {
        tcgetattr(STDIN_FILENO, &mut ORIGINAL_TIO);
        let mut new_tio = ORIGINAL_TIO;
        new_tio.c_lflag = new_tio.c_lflag & (!ICANON & !ECHO);
        tcsetattr(STDIN_FILENO, TCSANOW, &new_tio);
    };
}

pub fn restore_input_buffering() {
    unsafe { tcsetattr(STDIN_FILENO, TCSANOW, &ORIGINAL_TIO) };
}

pub fn get_char() -> u8 {
    unsafe { 
        let c = getchar(); 
        return c as u8;
    };
}

#[link(name = "c")]
extern "C" {
    pub fn tcgetattr(fd: c_int, termios_p: *mut termios) -> c_int;
    pub fn tcsetattr(fd: c_int, optional_actions: c_int, termios_p: *const termios) -> c_int;
    pub fn getchar() -> c_int;
}

