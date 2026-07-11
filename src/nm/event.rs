#[derive(Debug)]
#[expect(dead_code)]
pub enum Event {
    UploadSpeed { bytes_per_sec: u64 },
    DownloadSpeed { bytes_per_sec: u64 },
    NetworkSsidAndStrength { ssid: String, strength: u8 },
}
