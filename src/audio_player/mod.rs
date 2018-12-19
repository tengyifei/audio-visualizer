use chan;
use portaudio as pa;
use std::{thread, time};

use common_defs::AUDIO_PACKET_SIZE;

// Constants:
const NUM_CHANNELS: i32 = 2;
const FRAMES_PER_BUFFER: u32 = AUDIO_PACKET_SIZE as u32 * 4;
const BUFFER_SECONDS: f64 = 0.100; // Buffer samples for 100ms -- reduces chances of underrun
const SAMPLE_RATE: f64 = 44100.0;

/// "Run" the audio thread
/// Probably want to run this in a separate thread and send samples over a channel.
pub fn run(
    recv_audio_samples: chan::Receiver<[(i16, i16); AUDIO_PACKET_SIZE]>,
    send_graph_samples: chan::Sender<[(i16, i16); AUDIO_PACKET_SIZE]>,
) -> Result<(), pa::Error> {
    // Sleep a little so we don't underrun our audio buffer (probably not even needed but whatever):
    thread::sleep(time::Duration::new(0, 1_000_000));

    // Fire up ye olde PortAudio:
    let pa = try!(pa::PortAudio::new());

    // Set up our settings - set a buffer amount to try to reduce underruns:
    let mut settings =
        try!(pa.default_output_stream_settings(NUM_CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER));
    settings.params.suggested_latency = BUFFER_SECONDS;

    // This callback function will be called by PortAudio when it needs more audio samples.
    // It may be called at interrupt level on some machines, so don't do anything that could mess
    // up the system like dynamic resource allocation or I/O. (although doing so seems to be fine on
    // my machine...?)
    //
    // The job of this callback is to fill up the buffer that PortAudio tells us to fill up.
    // Each "frame" represents one sample for each channel that we have, so we need to put a total
    // of (NUM_CHANNELS * frames) samples into the buffer.
    // The samples are "interleaved" by default, so the structure of buffer looks like:
    // [ch0_sample0, ch1_sample0, ch0_sample1, ch1_sample1, ch0_sample2, ch1_sample2, ...]
    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, .. }| {
        let mut i = 0;
        while i < frames * 2 {
            match recv_audio_samples.recv() {
                Some(arr) => {
                    for pair in arr.iter() {
                        buffer[i] = (pair.0 as f32) / 32768.0;
                        buffer[i + 1] = (pair.1 as f32) / 32768.0;
                        i += 2;
                    }
                    send_graph_samples.send(arr.clone());
                }
                None => {
                    // Something...
                }
            };
        }
        pa::Continue
    };

    // Now that we have the settings and the callback function set up, we can finally open the
    // stream, through which we will actually play audio:
    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));

    // And now that we have the stream, we can start playing sounds!
    try!(stream.start());

    thread::sleep(time::Duration::new(0, 1_000_000_000));

    // We're using PortAudio in non-blocking mode, so execution will fall through immediately.
    // Sleep to make sure we keep playing audio
    loop {
        thread::sleep(time::Duration::new(1, 0));
    }

    // We're done playing, gracefully shut down the stream:
    // try!(stream.stop());
    // try!(stream.close());

    // Ok(())
}
