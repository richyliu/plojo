use serialport::{SerialPort, SerialPortSettings};
use std::{error::Error, io::ErrorKind, thread, time::Duration};

pub struct SerialMachine {
    /// How long to wait before trying to read from serial machine again
    read_rate: u64,
    /// Size of buffer to read each time
    buf_size: usize,
    port: Box<dyn SerialPort>,
}

impl SerialMachine {
    pub fn new(port_name: String) -> Result<Self, Box<dyn Error>> {
        let port = serialport::open_with_settings(&port_name, &SerialPortSettings::default())?;

        Ok(Self {
            read_rate: 50,
            buf_size: 6,
            port,
        })
    }

    pub fn read(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let sleep_time = Duration::from_millis(self.read_rate);
        let mut serial_buf: Vec<u8> = vec![0; self.buf_size];

        loop {
            match self.port.read_exact(serial_buf.as_mut_slice()) {
                Ok(()) => {
                    // successfully read data
                    return Ok(serial_buf);
                }
                Err(e) => match e.kind() {
                    ErrorKind::TimedOut => {
                        // no data to read, wait before trying again
                        thread::sleep(sleep_time);
                    }
                    _ => {
                        return Err(Box::new(e));
                    }
                },
            }
        }
    }
}
