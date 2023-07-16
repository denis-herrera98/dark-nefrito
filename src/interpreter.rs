use vosk::{Model, Recognizer};
use hound::WavReader;
use std::fs::File;
use std::io::{self, Write, BufReader};

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
        // println!("{:#?}", recognizer.partial_result());
        writeln!(handle, "{:#?}", recognizer.partial_result())?;
        writeln!(stdout.lock(), "{:#?}", recognizer.partial_result())?;
    }

    println!("{:#?}", recognizer.final_result().multiple().unwrap());

    Ok(())
}

fn describe_wav(reader: WavReader<BufReader<File>>) {
    let sample_format = reader.spec();
    println!("Sample format: {:?}", sample_format);
}


pub fn translate_sample(sample: &[i16]) -> io::Result<()> {

    let model_path = "/home/denisherrera/Documents/models/vosk-model-es-0.42";

    let model = Model::new(model_path).expect("Failed to load model");
    let mut recognizer = Recognizer::new(&model, 44100.0).unwrap();
    // describe_wav(reader);
    recognizer.set_max_alternatives(10);
    recognizer.set_words(true);
    recognizer.set_partial_words(true);


    let file_path = "/home/denisherrera/Documents/alma-negra/data/results.txt";
    let file = File::create(file_path)?;

    // Redirect standard output to the file
    io::stdout().flush()?;
    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(file);

        recognizer.accept_waveform(sample);
        // println!("{:#?}", recognizer.partial_result());
        writeln!(handle, "{:#?}", recognizer.partial_result())?;
        writeln!(stdout.lock(), "{:#?}", recognizer.partial_result())?;

    println!("{:#?}", recognizer.final_result().multiple().unwrap());

    Ok(())
}
