

use acid_io::{Read, Write};
use core::num;
use core::prelude::rust_2021::*;
use alloc::vec;
use alloc::vec::Vec;

/// Represents the type of data being sent
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum DataType {
    Print = 0x00,
    Error = 0x01,
    KernelLog = 0x02,
}



/// Implements the CEROS serial protocol
#[derive(Default)]
pub struct CEROSSerial<T: Read + Write> {
    stream: T,
    buffer: Vec<u8>,
    pros_compat: bool,
}

impl<T: Read + Write> CEROSSerial<T> {
    /// Creates a new instance of CEROSSerial
    pub fn new(stream: T) -> CEROSSerial<T> {
        CEROSSerial {
            stream,
            buffer: Vec::new(),
            pros_compat: false
        }
    }

    /// Creates a new instance of CEROSSerial with PROS
    /// compatibility enabled
    pub fn new_pros(stream: T) -> CEROSSerial<T> {
        CEROSSerial {
            stream,
            buffer: Vec::new(),
            pros_compat: true
        }
    }

    /// Creates a new serial packet
    pub fn create_serial_packet(&self, data_type: DataType, data: Vec<u8>) -> Vec<u8> {

        // Find the data to prepend to the vector based on
        // the packet type and PROS support
        let prepend: Vec<u8> = {
            if self.pros_compat {
                match data_type {
                    DataType::Print => b"sout".to_vec(),
                    DataType::Error => b"serr".to_vec(),
                    DataType::KernelLog => b"kdbg".to_vec(),
                    _ => {
                        // If PROS does not support it, then return none.
                        return Vec::new();
                    }
                }
            } else {
                // Magic number with data type
                vec![0x37u8, 0x31, 0x32, 0x32, data_type as u8]
            }
        };

        // Prepend the header to the data
        let mut packet = prepend;
        packet.extend(data);

        // COBS encode the data
        let mut out_data = vec![0u8; corncobs::max_encoded_len(packet.len())];
        let _size = corncobs::encode_buf(&packet, &mut out_data);

        // Return the data
        out_data
    }

    /// Parses a serial packet from an input vector
    pub fn parse_serial_packet(&self, data: Vec<u8>) -> (DataType, Vec<u8>) {

        // COBS decode the data
        let mut parsed_data = vec![0u8; data.len()];
        let num_decode = corncobs::decode_buf(&data, &mut parsed_data).unwrap_or(0);
        let data = parsed_data[..num_decode].to_vec();

        // If it starts with sout, serr, or kdbg it is a PROS packet
        if data.starts_with(b"sout") {
            (DataType::Print, data[4..].to_vec())
        } else if data.starts_with(b"serr") {
            (DataType::Error, data[4..].to_vec())
        } else if data.starts_with(b"kdbg") {
            (DataType::KernelLog, data[4..].to_vec())
        } else if data.starts_with(&[0x37, 0x31, 0x32, 0x32]) { // If it starts with the CEROS magic number, parse it as such
            // Find the data type
            let data_type = match data[4] {
                0x00 => DataType::Print,
                0x01 => DataType::Error,
                0x02 => DataType::KernelLog,
                _ => {
                    // If it is unrecognized, ignore
                    return (DataType::Print, Vec::new());
                }
            };

            // Get the rest of the bytes
            let data = data[5..].to_vec();

            

            (data_type, data)
        } else {
            // Otherwise return no data
            (DataType::Print, Vec::new())
        }
    }

    /// Reads in serial data
    pub fn read_data(&mut self) -> (DataType, Vec<u8>) {
        // Read in data so long as there are no 0x00 bytes in the buffer
        while !self.buffer.contains(&0x00) {
            let mut data = [0u8; 0xff];
            let size = self.stream.read(&mut data).unwrap();
            self.buffer.extend(&data[..size]);
        }

        // Find the index of the first 0x00 byte and split it off
        let pos = self.buffer.iter().position(|&r| r == 0x00).unwrap();
        let data: Vec<u8> = self.buffer.drain(0..pos).collect();
        
        // If there is still more data on the buffer, pop the last zero
        if !self.buffer.is_empty() {
            self.buffer.drain(0..1).for_each(drop);
        }

        // Parse and return the packet
        self.parse_serial_packet(data)
    }

    /// Writes serial data
    pub fn write_data(&mut self, data_type: DataType, data: Vec<u8>) -> usize {

        // Create the packet
        let packet = self.create_serial_packet(data_type, data);

        // Send it
        self.stream.write(&packet).unwrap()
    }
}