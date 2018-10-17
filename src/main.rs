extern crate ggez;
mod vm;
mod display;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::{channel, Sender, Receiver};

use vm::VM;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;
const N_PIXELS: usize = WIDTH * HEIGHT;
const N_BYTES: usize = 4 * N_PIXELS;

pub fn main() {

    let (snd, rcv) = channel::<Vec<u8>>();

    let controller = Arc::new(Mutex::new(0));

    // this thread just constantly copies the
    // screen into an image.
    let render_child =
        display::spawn_screen_thread(
            rcv,
            Arc::clone(&controller)
        );

    let mut cpu = VM::new(snd, controller);

    let prog = read_prog("test.zen");

    cpu.interpret(
        prog,
    );


    render_child.join();
}


fn read_prog(s: &'static str) -> Vec<u8> {
    let mut f = File::open(s).unwrap();
    let mut data = Vec::new();

    f.read_to_end(&mut data);
    return data
}
