use std::sync::{
    atomic::{AtomicBool, AtomicU8},
    Arc, RwLock,
};

use nvim_oxi as oxi;
use oxi::{Dictionary, Function};
use rodio::{OutputStream, Sink};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

mod audio;
mod errors;

#[oxi::module]
fn player() -> oxi::Result<Dictionary> {
    let counter = Arc::new(AtomicU8::new(0));
    let ping_inner = Function::from_fn(ping(counter));
    let harness_inner = Function::from_fn(harness);
    let (sender, receiver) = unbounded_channel();
    let start_inner = Function::from_fn(start(receiver));
    let stop_inner = Function::from_fn(stop(sender));

    let state = Arc::new(AtomicBool::new(false));
    let (_controller, antenna) = unbounded_channel();

    let player_starter = Function::from_fn(audio::start_playing(state, antenna));
    // let controller_inner = Function::from_fn(audio::controller(controller));

    Ok(Dictionary::from_iter([
        ("ping", ping_inner),
        ("harness", harness_inner),
        ("start", start_inner),
        ("stop", stop_inner),
        ("play", player_starter),
    ]))
}

fn ping(counter: Arc<AtomicU8>) -> impl Fn(()) -> Result<String, errors::OxiMorons> {
    move |_| {
        let i = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(format!("Hello, World! {}", i))
    }
}

fn start(receiver: UnboundedReceiver<()>) -> impl Fn(()) -> Result<String, errors::OxiMorons> {
    let recv = Arc::new(RwLock::new(receiver));
    move |_: ()| {
        let recv = recv.clone();
        std::thread::spawn(move || loop {
            let rx = recv.clone();
            match reqwest::blocking::get("http://localhost:8000/").ok() {
                None => break,
                _ => {}
            };
            match rx.write().unwrap().try_recv() {
                Ok(()) => break,
                _ => {}
            };
        });

        Ok::<_, errors::OxiMorons>("done".to_string())
    }
}

fn stop(sender: UnboundedSender<()>) -> impl Fn(()) -> Result<String, errors::OxiMorons> + 'static {
    move |_| {
        sender
            .send(())
            .map(|_| "Done".to_string())
            .map_err(|_| errors::OxiMorons::ComError("Failed while using channels"))
    }
}

fn harness(_: ()) -> Result<String, errors::OxiMorons> {
    std::thread::spawn(|| loop {
        match reqwest::blocking::get("http://localhost:8000/").ok() {
            Some(_) => {}
            None => {}
        }
        std::thread::sleep(std::time::Duration::from_secs(1))
    });

    Ok("done".to_string())
}

// mod build {
//     use std::env;
//     fn main() {
//         let path = std::path::Path::new("/usr/local/share/player");
//         if !path.exists() {
//             std::fs::create_dir_all("/usr/local/share/player/lua").unwrap();
//         }
//         let source = std::path::Path::new(&env::var("CRATE_OUT_DIR").unwrap()).join("libplayer.so");
//         let dst = std::path::Path::new("/usr/local/share/player/lua/player.so");
//         std::fs::copy(source, dst).unwrap();
//     }
// }
