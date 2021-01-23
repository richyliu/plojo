use plojo_core::{Machine, Stroke};
use serialport::{available_ports, SerialPortType};
use std::error::Error;

mod machine;
mod raw_stroke;

use machine::SerialMachine;

pub struct GeminiprMachine {
    machine: SerialMachine,
}

impl GeminiprMachine {
    pub fn new(config_port: &str) -> Result<Self, Box<dyn Error>> {
        let machine = SerialMachine::new(config_port)?;
        Ok(Self { machine })
    }
}

impl Machine for GeminiprMachine {
    fn read(&mut self) -> Result<Stroke, Box<dyn Error>> {
        self.machine.read().map(|raw| raw_stroke::parse_raw(&raw))
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
            eprintln!("[ERR] Could not get available ports: {:?}", e);
        }
    }
}

pub fn get_georgi_port() -> Option<String> {
    match available_ports() {
        Ok(ports) => {
            for p in ports {
                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        if info.manufacturer == Some("g Heavy Industries".to_string()) {
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
