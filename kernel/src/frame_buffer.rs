use bootloader_api::info::FrameBufferInfo;
use conquer_once::spin::OnceCell;
use spinning_top::Spinlock;

pub static CONSOLE: OnceCell<Spinlock<Console>> = OnceCell::uninit();

const BORDER_PADDING: usize = 1;

pub struct Console {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl Console {
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut console = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
        };
        console.clear();
        console
    }

    pub fn clear(&mut self) {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;
        self.framebuffer.fill(0);
    }
}
