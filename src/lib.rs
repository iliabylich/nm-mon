#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    UploadSpeed { bytes_per_sec: u64 },
    DownloadSpeed { bytes_per_sec: u64 },
    SsidAndStrength { ssid: String, strength: u8 },
}

#[allow(dead_code)]
impl Event {
    pub const SERIALIZED_LENGTH: usize = 32;
    const MAX_SSID_BYTESIZE: usize = 20;

    #[expect(
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::cast_possible_truncation
    )]
    pub fn serialize(&self) -> [u8; Self::SERIALIZED_LENGTH] {
        let mut out = [0; Self::SERIALIZED_LENGTH];

        match self {
            Self::UploadSpeed { bytes_per_sec } => {
                out[0] = 1;
                out[8..16].copy_from_slice(&bytes_per_sec.to_be_bytes());
            }
            Self::DownloadSpeed { bytes_per_sec } => {
                out[0] = 2;
                out[8..16].copy_from_slice(&bytes_per_sec.to_be_bytes());
            }
            Self::SsidAndStrength { ssid, strength } => {
                out[0] = 3;
                out[1] = *strength;

                let mut ssid_bytes_written = 0;
                for c in ssid.chars() {
                    let len = c.len_utf8();
                    assert!(len <= 4);
                    if ssid_bytes_written + len >= Self::MAX_SSID_BYTESIZE {
                        break;
                    }

                    let mut buf = [0; 4];
                    c.encode_utf8(&mut buf);
                    let buf = &buf[..len];

                    let start = 8 + ssid_bytes_written;
                    let end = start + len;
                    out[start..end].copy_from_slice(buf);

                    ssid_bytes_written += buf.len();
                }

                out[4..8].copy_from_slice(&(ssid_bytes_written as u32).to_be_bytes());
            }
        }

        out
    }

    pub fn deserialize(buf: [u8; Event::SERIALIZED_LENGTH]) -> Event {
        match buf[0] {
            1 => Event::UploadSpeed {
                bytes_per_sec: u64::from_be_bytes([
                    buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
                ]),
            },

            2 => Event::DownloadSpeed {
                bytes_per_sec: u64::from_be_bytes([
                    buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
                ]),
            },

            3 => Event::SsidAndStrength {
                ssid: {
                    let len = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
                    let bytes = &buf[8..][..len];
                    core::str::from_utf8(bytes).unwrap().to_string()
                },
                strength: buf[1],
            },

            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Event;

    #[test]
    fn test_upload_speed() {
        let e = Event::UploadSpeed {
            bytes_per_sec: 0x0123456789ABCDEF,
        };
        assert_eq!(Event::deserialize(e.serialize()), e);
    }

    #[test]
    fn test_download_speed() {
        let e = Event::DownloadSpeed {
            bytes_per_sec: 0x0123456789ABCDEF,
        };
        assert_eq!(Event::deserialize(e.serialize()), e);
    }

    #[test]
    fn test_ssid_and_strength() {
        let e = Event::SsidAndStrength {
            ssid: String::from("foo"),
            strength: 42,
        };
        assert_eq!(Event::deserialize(e.serialize()), e);
    }

    #[test]
    fn test_ssid_and_strength_truncated() {
        assert_eq!('ᔘ'.len_utf8(), 3);

        let full = Event::SsidAndStrength {
            ssid: "ᔘ".repeat(8),
            strength: 10,
        };
        let truncated = Event::SsidAndStrength {
            ssid: "ᔘ".repeat(6),
            strength: 10,
        };
        assert_eq!(Event::deserialize(full.serialize()), truncated);
    }
}
