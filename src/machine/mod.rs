use serialport::{available_ports, SerialPortSettings, SerialPortType};
use std::{any::Any, thread, time::Duration};

pub mod raw_stroke;

pub struct SerialMachine {
    // how often to poll for reads
    read_rate: u64,
    buf_size: usize,
    port_name: String,
    serialport_settings: SerialPortSettings,
}

impl Default for SerialMachine {
    fn default() -> Self {
        Self {
            read_rate: 10,
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

    pub fn with_baud_rate(self, baud_rate: u32) -> Self {
        Self {
            serialport_settings: SerialPortSettings {
                baud_rate,
                ..self.serialport_settings
            },
            ..self
        }
    }

    pub fn listen<T, U>(&self, action: T, initial_state: U)
    where
        T: Fn(&Vec<u8>, U) -> U,
        U: Any,
    {
        let port = serialport::open_with_settings(&self.port_name, &self.serialport_settings);

        let sleep_time = Duration::from_millis(self.read_rate);
        let mut serial_buf: Vec<u8> = vec![0; self.buf_size];

        match port {
            Ok(mut port) => {
                println!(
                    "Receiving data on {} at {} baud:",
                    &self.port_name, &self.serialport_settings.baud_rate
                );
                let mut state = initial_state;

                loop {
                    match port.read_exact(serial_buf.as_mut_slice()) {
                        Ok(()) => {
                            state = action(&serial_buf, state);
                        }
                        Err(e) => match e.kind() {
                            std::io::ErrorKind::TimedOut => {
                                // just a timeout (no data to read), ignore it
                            }
                            _ => {
                                eprintln!("err: {:?}", e);
                            }
                        },
                    }

                    thread::sleep(sleep_time);
                }
            }
            Err(e) => {
                eprintln!("Failed to open \"{}\". Error: {}", self.port_name, e);
            }
        }
    }

    pub fn print_available_ports() {
        match available_ports() {
            Ok(ports) => {
                match ports.len() {
                    0 => println!("No ports found."),
                    1 => println!("Found 1 port:"),
                    n => println!("Found {} ports:", n),
                };
                for p in ports {
                    println!("  {}", p.port_name);
                    match p.port_type {
                        SerialPortType::UsbPort(info) => {
                            println!("    Type: USB");
                            println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
                            println!(
                                "     Serial Number: {}",
                                info.serial_number.as_ref().map_or("", String::as_str)
                            );
                            println!(
                                "      Manufacturer: {}",
                                info.manufacturer.as_ref().map_or("", String::as_str)
                            );
                            println!(
                                "           Product: {}",
                                info.product.as_ref().map_or("", String::as_str)
                            );
                        }
                        SerialPortType::BluetoothPort => {
                            println!("    Type: Bluetooth");
                        }
                        SerialPortType::PciPort => {
                            println!("    Type: PCI");
                        }
                        SerialPortType::Unknown => {
                            println!("    Type: Unknown");
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
                eprintln!("Error listing serial ports");
            }
        }
    }

    pub fn get_georgi_port() -> Option<String> {
        match available_ports() {
            Ok(ports) => {
                for p in ports {
                    match p.port_type {
                        SerialPortType::UsbPort(info) => {
                            if info.product == Some("Georgi".to_string()) {
                                return Some(p.port_name);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(_) => {}
        }

        None
    }
}
