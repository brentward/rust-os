use atags::raw;
use std::slice;
use std::str;

pub use atags::raw::{Core, Mem};

/// An ATAG.
#[derive(Debug, Copy, Clone)]
pub enum Atag {
    Core(raw::Core),
    Mem(raw::Mem),
    Cmd(&'static str),
    Unknown(u32),
    None
}

impl Atag {
    /// Returns `Some` if this is a `Core` ATAG. Otherwise returns `None`.
    pub fn core(self) -> Option<Core> {
        match self {
            Atag::Core(core ) => Some(core),
            _ => None,
        }
    }

    /// Returns `Some` if this is a `Mem` ATAG. Otherwise returns `None`.
    pub fn mem(self) -> Option<Mem> {
        match self {
            Atag::Mem(mem ) => Some(mem),
            _ => None,
        }
    }

    /// Returns `Some` with the command line string if this is a `Cmd` ATAG.
    /// Otherwise returns `None`.
    pub fn cmd(self) -> Option<&'static str> {
        match self {
            Atag::Cmd(str ) => Some(str),
            _ => None,
        }
    }
}

// FIXME: Implement `From<raw::Core>`, `From<raw::Mem>`, and `From<&raw::Cmd>`
// for `Atag`. These implementations should be used by the `From<&raw::Atag> for
// Atag` implementation below.
impl<'a> From<&'a raw::Core> for Atag {
    fn from(core: &raw::Core) -> Atag {
        Atag::Core(*core)
    }
}

impl<'a> From<&'a raw::Mem> for Atag {
    fn from(mem: &raw::Mem) -> Atag {
        Atag::Mem(*mem)
    }
}

impl<'a> From<&'a raw::Cmd> for Atag {
    fn from(cmd: &raw::Cmd) -> Atag {
        let mut size = 0usize;
        let address = &cmd.cmd  as *const u8;
        unsafe {
            while *address.add(size) != 0 {
                size += 1;
            }
        }
        let cmd_slice = unsafe {slice::from_raw_parts(address, size)};
        let cmd_str = str::from_utf8(cmd_slice).expect("Cmd sring failed from utf8");
        Atag::Cmd(cmd_str)
    }
}

impl<'a> From<&'a raw::Atag> for Atag {
    fn from(atag: &raw::Atag) -> Atag {
        // FIXME: Complete the implementation below.

        unsafe {
            match (atag.tag, &atag.kind) {
                (raw::Atag::CORE, &raw::Kind { core }) => Atag::from(&core),
                (raw::Atag::MEM, &raw::Kind { mem }) => Atag::from(&mem),
                (raw::Atag::CMDLINE, &raw::Kind { ref cmd }) => Atag::from(cmd),
                (raw::Atag::NONE, _) => Atag::None,
                (id, _) => Atag::Unknown(id),
            }
        }
    }
}
