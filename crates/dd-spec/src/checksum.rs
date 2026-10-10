//! Named checksum catalogue (spec §7.5). Parameters from the CRC RevEng catalogue.

/// CRC parameters (Rocksoft model).
#[derive(Debug, Clone, Copy)]
struct Crc {
    width: u32,
    poly: u64,
    init: u64,
    refin: bool,
    refout: bool,
    xorout: u64,
}

const fn crc(width: u32, poly: u64, init: u64, refin: bool, refout: bool, xorout: u64) -> Crc {
    Crc {
        width,
        poly,
        init,
        refin,
        refout,
        xorout,
    }
}

/// Every algorithm name a conforming validator must know.
pub const NAMES: &[&str] = &[
    "none",
    "sum8",
    "xor8",
    "crc8-smbus",
    "crc8-maxim",
    "crc8-sensirion",
    "crc16-ccitt-false",
    "crc16-modbus",
    "crc16-xmodem",
    "crc16-usb",
    "crc32-iso-hdlc",
    "crc32c",
];

fn params(name: &str) -> Option<Crc> {
    Some(match name {
        "crc8-smbus" => crc(8, 0x07, 0x00, false, false, 0x00),
        "crc8-maxim" => crc(8, 0x31, 0x00, true, true, 0x00),
        "crc8-sensirion" => crc(8, 0x31, 0xFF, false, false, 0x00),
        "crc16-ccitt-false" => crc(16, 0x1021, 0xFFFF, false, false, 0x0000),
        "crc16-modbus" => crc(16, 0x8005, 0xFFFF, true, true, 0x0000),
        "crc16-xmodem" => crc(16, 0x1021, 0x0000, false, false, 0x0000),
        "crc16-usb" => crc(16, 0x8005, 0xFFFF, true, true, 0xFFFF),
        "crc32-iso-hdlc" => crc(32, 0x04C1_1DB7, 0xFFFF_FFFF, true, true, 0xFFFF_FFFF),
        "crc32c" => crc(32, 0x1EDC_6F41, 0xFFFF_FFFF, true, true, 0xFFFF_FFFF),
        _ => return None,
    })
}

fn reflect(mut v: u64, bits: u32) -> u64 {
    let mut r = 0;
    for _ in 0..bits {
        r = (r << 1) | (v & 1);
        v >>= 1;
    }
    r
}

/// Compute checksum `name` over `data`; `None` if the name is unknown.
pub fn compute(name: &str, data: &[u8]) -> Option<u64> {
    match name {
        "none" => return Some(0),
        "sum8" => return Some(data.iter().fold(0u8, |a, b| a.wrapping_add(*b)).into()),
        "xor8" => return Some(data.iter().fold(0u8, |a, b| a ^ b).into()),
        _ => {}
    }
    let p = params(name)?;
    let top = 1u64 << (p.width - 1);
    let mask = if p.width == 64 {
        u64::MAX
    } else {
        (1u64 << p.width) - 1
    };
    let mut reg = p.init;
    for &byte in data {
        let b = if p.refin {
            reflect(u64::from(byte), 8)
        } else {
            u64::from(byte)
        };
        reg ^= b << (p.width - 8);
        for _ in 0..8 {
            reg = if reg & top != 0 {
                (reg << 1) ^ p.poly
            } else {
                reg << 1
            };
            reg &= mask;
        }
    }
    if p.refout {
        reg = reflect(reg, p.width);
    }
    Some((reg ^ p.xorout) & mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveng_check_values() {
        let c = |n| compute(n, b"123456789").unwrap();
        assert_eq!(c("crc8-smbus"), 0xF4);
        assert_eq!(c("crc8-maxim"), 0xA1);
        assert_eq!(c("crc8-sensirion"), 0xF7);
        assert_eq!(c("crc16-ccitt-false"), 0x29B1);
        assert_eq!(c("crc16-modbus"), 0x4B37);
        assert_eq!(c("crc16-xmodem"), 0x31C3);
        assert_eq!(c("crc16-usb"), 0xB4C8);
        assert_eq!(c("crc32-iso-hdlc"), 0xCBF4_3926);
        assert_eq!(c("crc32c"), 0xE306_9283);
        assert_eq!(c("sum8"), 0xDD);
        assert_eq!(c("xor8"), 0x31);
        assert!(compute("crc7", b"").is_none());
    }

    #[test]
    fn every_listed_name_is_known() {
        for n in NAMES {
            assert!(compute(n, b"x").is_some(), "{n}");
        }
    }
}
