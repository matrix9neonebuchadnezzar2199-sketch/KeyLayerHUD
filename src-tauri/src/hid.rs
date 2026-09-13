use crate::layer_report::{build_layer_query_packet, parse_layer_report, LayerReport};
use crate::via::{
    buffer_chunk_to_keycodes, build_hid_command, flat_to_matrix, parse_keymap_buffer_chunk,
    shift_to_16bit, CMD_DYNAMIC_KEYMAP_GET_BUFFER, CMD_DYNAMIC_KEYMAP_GET_LAYER_COUNT,
    CMD_GET_PROTOCOL_VERSION, PROTOCOL_BETA, VIA_USAGE, VIA_USAGE_PAGE,
};
use hidapi::{HidApi, HidDevice};
use std::ffi::CString;
use std::time::Duration;

pub struct ViaDevice {
    device: HidDevice,
    path: String,
}

impl ViaDevice {
    pub fn enumerate(api: &HidApi) -> Vec<(String, u16, u16)> {
        api.device_list()
            .filter(|d| d.usage_page() == VIA_USAGE_PAGE && d.usage() == VIA_USAGE)
            .map(|d| {
                (
                    d.path().to_string_lossy().to_string(),
                    d.vendor_id(),
                    d.product_id(),
                )
            })
            .collect()
    }

    pub fn open(api: &HidApi, path: &str) -> Result<Self, String> {
        let cpath = CString::new(path).map_err(|e| format!("invalid HID path: {e}"))?;
        let device = api
            .open_path(&cpath)
            .map_err(|e| format!("HID open failed: {e}"))?;
        device
            .set_blocking_mode(true)
            .map_err(|e| format!("blocking mode: {e}"))?;
        Ok(Self {
            device,
            path: path.to_string(),
        })
    }

    fn write_read(&self, command: u8, payload: &[u8]) -> Result<Vec<u8>, String> {
        let packet = build_hid_command(command, payload);
        self.device
            .write(&packet)
            .map_err(|e| format!("HID write failed: {e}"))?;
        let mut buf = [0u8; 64];
        let len = self
            .device
            .read_timeout(&mut buf, 500)
            .map_err(|e| format!("HID read failed: {e}"))?;
        Ok(buf[..len].to_vec())
    }

    pub fn protocol_version(&self) -> Result<u16, String> {
        let resp = self.write_read(CMD_GET_PROTOCOL_VERSION, &[])?;
        if resp.len() < 5 {
            return Err("short protocol version response".into());
        }
        Ok(shift_to_16bit(resp[3], resp[4]))
    }

    pub fn layer_count(&self) -> Result<u8, String> {
        let version = self.protocol_version()?;
        if version < PROTOCOL_BETA {
            return Ok(4);
        }
        let resp = self.write_read(CMD_DYNAMIC_KEYMAP_GET_LAYER_COUNT, &[])?;
        if resp.len() < 4 {
            return Err("short layer count response".into());
        }
        Ok(resp[3])
    }

    pub fn read_layer_matrix(
        &self,
        layer: u8,
        rows: u8,
        cols: u8,
    ) -> Result<Vec<Vec<u16>>, String> {
        let length = (rows as usize) * (cols as usize);
        let mut flat = Vec::with_capacity(length);
        let chunk_keycodes = 14;
        let mut offset = layer as u32 * length as u32 * 2;
        while flat.len() < length {
            let remaining = length - flat.len();
            let take = remaining.min(chunk_keycodes);
            let size = (take * 2) as u8;
            let payload = [
                (offset & 0xFF) as u8,
                ((offset >> 8) & 0xFF) as u8,
                size,
            ];
            let resp = self.write_read(CMD_DYNAMIC_KEYMAP_GET_BUFFER, &payload)?;
            let bytes = parse_keymap_buffer_chunk(&resp, size);
            flat.extend(buffer_chunk_to_keycodes(&bytes));
            offset += size as u32;
        }
        Ok(flat_to_matrix(&flat, rows, cols))
    }

    pub fn query_layer_report(&self) -> Result<LayerReport, String> {
        let packet = build_layer_query_packet();
        self.device
            .write(&packet)
            .map_err(|e| format!("layer query write: {e}"))?;
        let mut buf = [0u8; 64];
        let len = self
            .device
            .read_timeout(&mut buf, 300)
            .map_err(|e| format!("layer query read: {e}"))?;
        parse_layer_report(&buf[..len]).ok_or_else(|| "invalid layer report".into())
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

pub fn init_hid() -> Result<HidApi, String> {
    HidApi::new().map_err(|e| format!("hidapi init: {e}"))
}

pub fn sleep_poll() {
    std::thread::sleep(Duration::from_millis(80));
}
