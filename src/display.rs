// i have no idea if this is thebest way to render
// an array of rgb values.
// it would be nicer to set up a channel if i could
// figure out how t odraw individual pixels...

use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Sender, Receiver};

use ggez::conf;
use ggez::event;
use ggez::event::{Button, Axis};
use ggez::graphics::{self, Image, Color, FilterMode};
use ggez::{Context, GameResult};

pub type BufferRecv = Receiver<Vec<u8>>;
pub type BufferSend = Sender<Vec<u8>>;
pub type ControllerHandle = Arc<Mutex<u8>>;


pub struct ZenState {
    screen: BufferRecv,
    controller: Arc<Mutex<u8>>,
    buf: [u8; ::N_BYTES],
    width: u16,
    height: u16,
}

pub fn spawn_screen_thread(
    screen:     BufferRecv,
    controller: ControllerHandle,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut c = conf::Conf::new();
        c.window_setup.resizable = false;
        c.window_setup.title = "Zen".to_string();
        c.window_mode.width = ::WIDTH as u32;
        c.window_mode.height = ::HEIGHT as u32;

        let ctx = &mut Context::load_from_conf(
            "Zen ~", "s. haug", c
            ).unwrap();

        let state =
            &mut ZenState::new(
                screen, controller
            );

        event::run(ctx, state).unwrap();
    })
}

impl ZenState {
    pub fn new(
        scr: Receiver<Vec<u8>>,
        con: Arc<Mutex<u8>>,
    ) -> ZenState {
        ZenState {
            screen: scr,
            controller: con,
            buf: [0; ::N_BYTES],
            width: ::WIDTH as u16,
            height: ::HEIGHT as u16,
        }
    }
}

impl event::EventHandler for ZenState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {

        if let Ok(buf) = self.screen.try_recv() {
            for i in 0..::N_BYTES {
                self.buf[i] = buf[i];
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // first lets render to our canvas
        let mut display =
            Image::from_rgba8(
                ctx, self.width, self.height, &self.buf
            ).unwrap();

        display.set_filter(FilterMode::Nearest);

        let origin = graphics::Point2::new(0.0, 0.0);
        graphics::draw(ctx, &display, origin, 0.0);

        let window_size = graphics::get_size(ctx);
        graphics::present(ctx);

        Ok(())
    }

    fn controller_button_down_event(
        &mut self,
        _ctx: &mut Context,
        btn: Button,
        _instance_id: i32
    ) {
        use ggez::event::Button::*;
        let mask = match btn {
            A         => 0b00000001,
            B         => 0b00000010,
            X         => 0b00000100,
            Y         => 0b00001000,
            DPadLeft  => 0b00010000,
            DPadRight => 0b00100000,
            DPadUp    => 0b01000000,
            DPadDown  => 0b10000000,
            _         => 0b00000000,
        };

        let mut g = self.controller.lock().unwrap();
        *g |= mask;
    }

    fn controller_button_up_event(
        &mut self,
        _ctx: &mut Context,
        btn: Button,
        _instance_id: i32
    ) {
        use ggez::event::Button::*;
        let mask = match btn {
            A         => 0b00000001,
            B         => 0b00000010,
            X         => 0b00000100,
            Y         => 0b00001000,
            DPadDown  => 0b00010000,
            DPadRight => 0b00100000,
            DPadLeft  => 0b01000000,
            DPadUp    => 0b10000000,
            _         => 0b00000000,
        };

        let mut g = self.controller.lock().unwrap();
        *g &= !mask;
    }

    fn controller_axis_event(
        &mut self,
        _ctx: &mut Context,
        _axis: Axis,
        _value: i16,
        _instance_id: i32
    ) {
        // dead zone
        if _value > 8000 || _value < -8000 {
        }
    }
}


pub fn set(pixels: &mut [u8], x: usize, y: usize, rgb: u8) {
    let r: u8 = 36 * ((rgb & 0b11100000) >> 5);
    let g: u8 = 36 * ((rgb & 0b00011100) >> 3);
    let b: u8 = 85 * (rgb & 0b00000011);
    let i = (x + y * 160) * 4;
    pixels[i + 0] = r;
    pixels[i + 1] = g;
    pixels[i + 2] = b;
    pixels[i + 3] = 255;
}
