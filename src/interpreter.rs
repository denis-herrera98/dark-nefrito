use vosk::{Model, Recognizer};
use hound::WavReader;
use std::fs::File;
use std::io::{self, Write, BufReader};
use hound::WavWriter;
use std::f64::consts::PI;

pub fn write_example() {
    let mut wav_write = WavWriter::create("/home/denisherrera/Documents/alma-negra/data/loco.wav",
        hound::WavSpec {
        bits_per_sample: 16,
        channels: 1,
        sample_format: hound::SampleFormat::Int,
        sample_rate: 44100,
    }).unwrap();

    
    // Generate audio data (e.g., a simple sine wave).
    for t in (0..44100).map(|x| x as f64 / 44100.0) {
        let sample_left = (t * 440.0 * 2.0 * PI).sin();
        let sample_right = (t * 880.0 * 2.0 * PI).sin();
        wav_write.write_sample((sample_left * i16::MAX as f64) as i16).ok();
        wav_write.write_sample((sample_right * i16::MAX as f64) as i16).ok();
    }

    wav_write.finalize().ok();
}

pub fn start_model() -> io::Result<()> {

    let audio_path = "/home/denisherrera/Documents/alma-negra/data/loco.wav";
    let model_path = "/home/denisherrera/Documents/models/vosk-model-es-0.42";

    // Load WAV
    let mut reader = WavReader::open(audio_path).expect("Could not create the WAV reader");
    let samples = reader
        .samples()
        .collect::<hound::Result<Vec<i16>>>()
        .expect("Could not read WAV file");
    // Load the Vosk model
    let model = Model::new(model_path).expect("Failed to load model");
    let mut recognizer = Recognizer::new(&model, 44100.0).unwrap();
    // describe_wav(reader);
    recognizer.set_max_alternatives(10);
    recognizer.set_words(true);
    recognizer.set_partial_words(true);


    let file_path = "/home/denisherrera/Documents/alma-negra/data/results.txt";
    let mut file = File::create(file_path)?;

    // Redirect standard output to the file
    io::stdout().flush()?;
    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(file);

    for sample in samples.chunks(4000) {
        recognizer.accept_waveform(sample);
        println!("SAMPLE SAMPLE SAMPLE {:?}", sample.len());
        // println!("{:#?}", recognizer.partial_result());
      //  writeln!(handle, "{:#?}", recognizer.partial_result())?;
      //  writeln!(stdout.lock(), "{:#?}", recognizer.partial_result())?;
    }

    println!("{:#?}", recognizer.final_result().multiple().unwrap());

    Ok(())
}

fn describe_wav(reader: WavReader<BufReader<File>>) {
    let sample_format = reader.spec();
    println!("Sample format: {:?}", sample_format);
}


pub fn translate_sample(sample: &[i16]) -> io::Result<()> {

    let audio_path = "/home/denisherrera/Documents/alma-negra/data/loco.wav";
    let model_path = "/home/denisherrera/Documents/models/vosk-model-es-0.42";

    let model = Model::new(model_path).expect("Failed to load model");
    let mut recognizer = Recognizer::new(&model, 44100.0).unwrap();
    // describe_wav(reader);
    recognizer.set_max_alternatives(10);
    recognizer.set_words(true);
    recognizer.set_partial_words(true);


//    let mut file = File::create(file_path)?;
//    let mut file = File::create(file_path)?;

    // Redirect standard output to the file
    io::stdout().flush()?;
    let stdout = io::stdout();
//    let mut handle = io::BufWriter::new(file);

        recognizer.accept_waveform(sample);
        // println!("{:#?}", recognizer.partial_result());
 //       writeln!(handle, "{:#?}", recognizer.partial_result())?;
        writeln!(stdout.lock(), "{:#?}", recognizer.partial_result())?;

    println!("{:#?}", recognizer.final_result().multiple().unwrap());

    Ok(())
}
