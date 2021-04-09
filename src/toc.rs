use num_enum::IntoPrimitive;

#[derive(IntoPrimitive, Debug)]
#[repr(u32)]
pub enum TocFlag {
    MetaData = 1 << 1,
    NewObjList = 1 << 2,
    RawData = 1 << 3,
    InterleavedData = 1 << 5,
    BigEndian = 1 << 6,
    DaqMxRawData = 1 << 7,
}

#[derive(Debug)]
pub struct TocMask {
    flags: u32,
}

impl TocMask {
    pub fn from_flags(flags: u32) -> TocMask {
        TocMask { flags }
    }

    pub fn has_flag(&self, flag: TocFlag) -> bool {
        let flag_val: u32 = flag.into();
        (self.flags & flag_val) == flag_val
    }
}

impl std::fmt::Display for TocMask {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.flags.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn can_check_flags() {
        let toc_mask = TocMask::from_flags(14);

        assert_eq!(toc_mask.has_flag(TocFlag::MetaData), true);
        assert_eq!(toc_mask.has_flag(TocFlag::NewObjList), true);
        assert_eq!(toc_mask.has_flag(TocFlag::RawData), true);

        assert_eq!(toc_mask.has_flag(TocFlag::InterleavedData), false);
        assert_eq!(toc_mask.has_flag(TocFlag::BigEndian), false);
        assert_eq!(toc_mask.has_flag(TocFlag::DaqMxRawData), false);
    }
}
