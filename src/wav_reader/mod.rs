use hound;
use chan::Sender;

use common_defs::AUDIO_PACKET_SIZE;

/// Read samples from a .wav file
pub fn read_samples(filename: &str, send_audio_samples: Sender<[(i16, i16); AUDIO_PACKET_SIZE]>) {
    // Get an iterator over samples in the .wav file:
    let mut reader = hound::WavReader::open(filename).unwrap();
    let mut sample_iterator = reader.samples::<i16>();

    let mut samples: [(i16, i16); AUDIO_PACKET_SIZE] = [(0, 0); AUDIO_PACKET_SIZE];
    let mut i = 0;
    loop {
        let left = match sample_iterator.next() {
            Some(Ok(t)) => t,
            _ => break,
        };
        let right = match sample_iterator.next() {
            Some(Ok(t)) => t,
            _ => break,
        };
        samples[i] = (left, right);
        if i == AUDIO_PACKET_SIZE - 1 {
            send_audio_samples.send(samples.clone());
            i = 0;
        } else {
            i += 1;
        }
    }
    println!("End of file.");
}
