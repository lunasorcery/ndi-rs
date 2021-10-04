use std::{
    sync::{mpsc::channel, Arc},
    thread,
    time::Instant,
};

fn main() {
    ndi::load_library_default().unwrap();
    ndi::initialize().unwrap();

    let find = ndi::FindBuilder::new().build().unwrap();

    let sources = find.current_sources(1000).unwrap();

    let mut recv = ndi::RecvBuilder::new().build().unwrap();
    println!("Connecting to the first source: {}", sources[0].get_name());
    recv.connect(&sources[0]);

    let recv_arc = Arc::new(recv);
    let video_arc = recv_arc.clone();
    let audio_arc = video_arc.clone();

    let (video_tx, video_rx) = channel();
    thread::spawn(move || {
        println!("Running video capture in thread 1");
        let start = Instant::now();
        while Instant::now().duration_since(start).as_millis() < 10000 {
            let mut video_data = None;
            let response = video_arc.capture_video(&mut video_data, 1000);
            if response == ndi::FrameType::Video {
                if let Some(video) = video_data {
                    video_tx.send(video).unwrap()
                }
            }
        }
    });

    let (audio_tx, audio_rx) = channel();
    thread::spawn(move || {
        let recv = audio_arc.clone();
        println!("Running audio capture in thread 2");
        let start = Instant::now();
        while Instant::now().duration_since(start).as_millis() < 10000 {
            let mut audio_data = None;
            let response = recv.capture_audio(&mut audio_data, 1000);
            if response == ndi::FrameType::Audio {
                if let Some(audio) = audio_data {
                    audio_tx.send(audio).unwrap()
                }
            }
        }
    });

    let start = Instant::now();
    while Instant::now().duration_since(start).as_millis() < 1000 {
        if let Ok(video_data) = video_rx.recv().map_err(|e| e.to_string()) {
            println!(
                "Received video on main thread: {}x{}",
                video_data.width(),
                video_data.height()
            );
        }

        if let Ok(audio_data) = audio_rx.recv() {
            println!(
                "Received audio on main thread: {}",
                audio_data.no_channels()
            );
        }
    }

    unsafe {
        ndi::cleanup();
    }
}
