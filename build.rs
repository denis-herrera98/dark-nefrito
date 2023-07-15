fn main() {
    // Specify the library search path
    println!("cargo:rustc-link-search=native=/home/denisherrera/Downloads/vosk-0.3.45-py3-none-linux_x86_64/vosk");

    // Specify the Vosk library name
    println!("cargo:rustc-link-lib=vosk");

}
