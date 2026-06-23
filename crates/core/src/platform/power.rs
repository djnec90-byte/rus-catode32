//! Software reset / process exit. Firmware drops into the chip's reset path;
//! desktop exits the process.

#[cfg(not(feature = "desktop"))]
pub fn software_reset() -> ! {
    esp_hal::system::software_reset()
}

#[cfg(feature = "desktop")]
pub fn software_reset() -> ! {
    std::process::exit(0)
}
