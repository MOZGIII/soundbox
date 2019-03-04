extern crate cpal;

mod none_option;
use none_option::NoneOption;

fn main() -> Result<(), Box<std::error::Error>> {
    let gain: f32 = std::env::args()
        .nth(1)
        .map_or(1.0, |s| s.parse::<f32>().unwrap_or(1.0));

    // Setup the default input device and stream with the default input format.
    let input_device = cpal::default_input_device().ok_or(NoneOption)?;
    println!("Default input device: {}", input_device.name());

    let input_format = input_device.default_input_format()?;
    println!("Default input format: {:?}", input_format);

    let output_device = cpal::default_output_device().ok_or(NoneOption)?;
    println!("Default output device: {}", output_device.name());

    let output_format = output_device.default_output_format()?;
    println!("Default output format: {:?}", output_format);

    let event_loop = cpal::EventLoop::new();

    let input_stream_id = event_loop.build_input_stream(&input_device, &input_format)?;
    event_loop.play_stream(input_stream_id);

    let output_stream_id = event_loop.build_output_stream(&output_device, &output_format)?;
    event_loop.play_stream(output_stream_id);

    println!("Begin...");
    let working = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let working_consumer = working.clone();

    let io_buffer = std::collections::VecDeque::<f32>::with_capacity(30_000_000);
    let mu = std::sync::Mutex::new(io_buffer);

    std::thread::spawn(move || {
        event_loop.run(move |_stream_id, stream_data| {
            if !working_consumer.load(std::sync::atomic::Ordering::Relaxed) {
                let io_buffer = mu.lock().unwrap();
                dbg!(io_buffer.len());
                return;
            }

            match stream_data {
                cpal::StreamData::Input {
                    buffer: cpal::UnknownTypeInputBuffer::F32(buffer),
                } => {
                    let mut io_buffer = mu.lock().unwrap();
                    io_buffer.extend(buffer.into_iter());
                }

                cpal::StreamData::Output {
                    buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                } => {
                    let mut io_buffer = mu.lock().unwrap();
                    for elem in buffer.iter_mut() {
                        *elem = io_buffer.pop_front().unwrap_or(0.0) * gain;
                    }
                }

                _ => panic!("unhandled format"),
            }
        });
    });

    std::thread::sleep(std::time::Duration::from_secs(600));
    working.store(false, std::sync::atomic::Ordering::Relaxed);

    Ok(())
}
