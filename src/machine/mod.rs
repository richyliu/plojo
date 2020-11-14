use raw_stroke::{RawStroke, RawStrokeGeminipr};
use serialport::{available_ports, SerialPortType};
use std::{thread, time::Duration};

mod raw_stroke;

fn print_available_ports() {
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

const BAUD_RATE: u32 = 9600;
const READ_RATE: u64 = 10; // ms delay between reads
fn read_from_port<T>(port_name: &str, buf_size: usize, action: T)
where
    T: Fn(&Vec<u8>),
{
    let port = serialport::open(port_name);

    let sleep_time = Duration::from_millis(READ_RATE);
    let mut serial_buf: Vec<u8> = vec![0; buf_size];

    match port {
        Ok(mut port) => {
            println!("Receiving data on {} at {} baud:", &port_name, &BAUD_RATE);
            loop {
                match port.read_exact(serial_buf.as_mut_slice()) {
                    Ok(()) => action(&serial_buf),
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
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
        }
    }
}

pub fn run() {
    read_from_port("/dev/ttyACM0", 6, |raw| {
        let stroke = RawStrokeGeminipr::parse_raw(raw).to_stroke();
        println!("raw: {:?}, stroke: {:?}", raw, stroke);
    });
}
