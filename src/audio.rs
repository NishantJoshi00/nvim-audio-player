use std::{
    fs::{self, File, OpenOptions},
    io::BufReader,
    io::Write,
    os::unix::thread::JoinHandleExt,
    sync::{self, atomic::AtomicBool, Arc, RwLock},
    thread,
    time::Duration,
};

use rodio::{Decoder, OutputStream, Sink, Source};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::errors;

fn file_logger(data: impl ToString) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("/tmp/oxi-logs")
        .unwrap();
    writeln!(file, "{}", data.to_string()).expect("no logs, go yolo")
}

pub fn start_playing(
    state: sync::Arc<AtomicBool>,
    receiver: UnboundedReceiver<AudioAction>,
) -> impl Fn(()) -> Result<String, errors::OxiMorons> {
    let recv = Arc::new(RwLock::new(receiver));
    move |_| {
        let recv = recv.clone();
        if !state.load(sync::atomic::Ordering::Acquire) {
            state.store(true, sync::atomic::Ordering::Release);
            thread::spawn(move || {
                let inner_recv = recv.clone();
                file_logger("sync created");

                let (_stream, handle) = OutputStream::try_default().expect("speaker broke");
                let sink = Sink::try_new(&handle).expect("speaker on the left broke");
                let source = Decoder::new(BufReader::new(
                    File::open(format!(
                        "{}/.config/nvim/music/lofi.mp3",
                        std::env::var("HOME").expect("No home to go"),
                    ))
                    .unwrap(),
                ))
                .unwrap();

                // let source = rodio::source::SineWave::new(440.0)
                //     .take_duration(Duration::from_secs_f32(12.0))
                //     .amplify(0.80);

                // sink.sleep_until_end();
                // sink.play();
                sink.append(source);
                loop {
                    file_logger("song played");
                    match inner_recv
                        .write()
                        .ok()
                        .and_then(|mut inner| inner.blocking_recv())
                    {
                        Some(AudioAction::Close) | None => {
                            sink.stop();
                            break;
                        }
                        Some(AudioAction::Pause) => sink.pause(),
                        Some(AudioAction::Play) => sink.play(),
                    }
                }
            });
        }
        Ok("I think it started".to_string())
    }
}

#[allow(dead_code)]
pub enum AudioAction {
    Close,
    Pause,
    Play,
}
