use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};

pub struct FrameBufferWriter {
    fb: &'static mut [u8],
    fb_info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl FrameBufferWriter {
    pub fn new(fb: &'static mut FrameBuffer) -> Self {
        let mut writer = Self {
            fb_info: fb.info().clone(),
            fb: fb.buffer_mut(),
            x_pos: 0,
            y_pos: 0,
        };

        writer.clear();

        writer
    }

    pub fn clear(&mut self) {
        self.fb.fill(0);
        self.x_pos = 0;
        self.y_pos = 0;
    }

    pub fn write_pixel(&mut self, x: usize, y: usize) {
        let pos_offset = x + y * self.fb_info.stride;
        let bytes_per_pixel = self.fb_info.bytes_per_pixel;
        let byte_offset = pos_offset * bytes_per_pixel;
        let color = [255, 255, 255, 0];

        self.fb[byte_offset..byte_offset + bytes_per_pixel]
            .copy_from_slice(&color[..bytes_per_pixel]);
    }
}
