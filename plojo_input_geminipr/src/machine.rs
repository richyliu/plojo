use serialport::SerialPortSettings;
use std::{any::Any, io::ErrorKind, thread, time::Duration};

pub struct SerialMachine {
    /// How long to wait before trying to read from serial machine again
    read_rate: u64,
    buf_size: usize,
    port_name: String,
    serialport_settings: SerialPortSettings,
}

impl Default for SerialMachine {
    fn default() -> Self {
        Self {
            read_rate: 50,
            buf_size: 6,
            port_name: String::from(""),
            serialport_settings: SerialPortSettings::default(),
        }
    }
}

impl SerialMachine {
    pub fn new(port_name: String) -> Self {
        Self {
            port_name,
            ..Self::default()
        }
    }

    pub fn listen<T, U>(&self, on_stroke: T, state: &mut U)
    where
        T: Fn(&Vec<u8>, &mut U),
        U: Any,
    {
        let port = serialport::open_with_settings(&self.port_name, &self.serialport_settings);

        let sleep_time = Duration::from_millis(self.read_rate);
        let mut serial_buf: Vec<u8> = vec![0; self.buf_size];

        match port {
            Ok(mut port) => {
                // keep on reading as long as there's more data
                loop {
                    match port.read_exact(serial_buf.as_mut_slice()) {
                        Ok(()) => {
                            on_stroke(&serial_buf, state);
                        }
                        Err(e) => match e.kind() {
                            ErrorKind::TimedOut => {
                                // no more data to read, wait before trying again
                                thread::sleep(sleep_time);
                            }
                            ErrorKind::BrokenPipe => {
                                // broken pipe usually means the serial port disconnected
                                println!("Machine disconnected. Exiting.");
                                return;
                            }
                            _ => {
                                panic!("error reading: {}", e);
                            }
                        },
                    }
                }
            }
            Err(e) => {
                panic!("Failed to open \"{}\". Error: {}", self.port_name, e);
            }
        }
    }
}
