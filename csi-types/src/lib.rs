pub mod ser;
use ser::{SerCSI, ComplexDef};

use std::fs;
use std::io::Read;
use std::convert::TryInto;

use num::complex::Complex;

#[allow(non_upper_case_globals)]
pub const Kernel_CSI_ST_LEN: usize = 23;
pub const CSI_ST_LEN: usize = 23;

fn c_cond(cond: isize) -> bool {
    cond != 0
}

pub fn is_big_endian() -> bool {
    #[cfg(target_endian = "big")]
    {
        true
    }
    #[cfg(not(target_endian = "big"))]
    {
        false
    }
}

pub fn bit_convert(data: isize, maxbit: isize) -> isize {
    let mut d = data;
    if c_cond( d & (1 << (maxbit - 1)) )
    {
        /* negative */
        d -= (1 << maxbit);
    }
    return d;
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CSIStruct {
    tstamp: u64,         /* h/w assigned time stamp */
    
    channel: u16,        /* wireless channel (represented in Hz)*/
    chanBW: u8,         /* channel bandwidth (0->20MHz,1->40MHz)*/

    rate: u8,           /* transmission rate*/
    nr: u8,             /* number of receiving antenna*/
    nc: u8,             /* number of transmitting antenna*/
    num_tones: u8,      /* number of tones (subcarriers) */
    noise: u8,          /* noise floor (to be updated)*/

    phyerr: u8,          /* phy error code (set to 0 if correct)*/

    rssi: u8,         /*  rx frame RSSI */
    rssi_0: u8,       /*  rx frame RSSI [ctl, chain 0] */
    rssi_1: u8,       /*  rx frame RSSI [ctl, chain 1] */
    rssi_2: u8,       /*  rx frame RSSI [ctl, chain 2] */

    pub payload_len: u16,  /*  payload length (bytes) */
    csi_len: u16,      /*  csi data length (bytes) */
    buf_len: u16,      /*  data length in buffer */
}

impl CSIStruct {
    pub fn new() -> Self {
        Self {
            tstamp: 0,         /* h/w assigned time stamp */
            
            channel: 0,        /* wireless channel (represented in Hz)*/
            chanBW: 0,         /* channel bandwidth (0->20MHz,1->40MHz)*/

            rate: 0,           /* transmission rate*/
            nr: 0,             /* number of receiving antenna*/
            nc: 0,             /* number of transmitting antenna*/
            num_tones: 0,      /* number of tones (subcarriers) */
            noise: 0,          /* noise floor (to be updated)*/

            phyerr: 0,          /* phy error code (set to 0 if correct)*/

            rssi: 0,         /*  rx frame RSSI */
            rssi_0: 0,       /*  rx frame RSSI [ctl, chain 0] */
            rssi_1: 0,       /*  rx frame RSSI [ctl, chain 1] */
            rssi_2: 0,       /*  rx frame RSSI [ctl, chain 2] */

            payload_len: 0,  /*  payload length (bytes) */
            csi_len: 0,      /*  csi data length (bytes) */
            buf_len: 0,      /*  data length in buffer */
        }
    }
}

use serde::{Deserialize, Serialize};

pub struct CSI {
    file: fs::File,

    pub csi_matrix: Vec<Vec<Vec<Complex<isize>>>>,

    buf: Vec<u8>,
    data_buf: Vec<u8>,

    pub csi_status: CSIStruct,
}

impl CSI {
    /// Convert into serializable type
    pub fn to_ser(&self) -> SerCSI {
        let m = self.csi_matrix.clone();
        let mm = m.iter().map(
            |a| a.iter().map(
                |b| b.iter().map(
                    |x| ComplexDef { re: x.re, im: x.im }
                ).collect()
            ).collect()
        ).collect();
        
        SerCSI {
            csi_matrix: mm,
            csi_status: self.csi_status.clone(),
        }
    }

    pub fn with_file(fpath: &str) -> Self {
        Self {
            file: fs::File::open(fpath)
                .expect(&format!("Cannot open {} device", fpath)),
            csi_matrix: vec![vec![vec![Complex::new(0, 0); 114]; 3]; 3],
            buf: vec![0; 4096],
            data_buf: vec![0; 15000],
            csi_status: CSIStruct::new(),
        }
    }

    /// fill_matrix
    #[allow(unused_variables)]
    pub fn fill_matrix(
        &mut self,
        csi_addr: &[u8],
        nr: usize,
        nc: usize,
        num_tones: usize,
    ) {
        let k: u8;

        let mut bits_left: u8 = 16;

        let bitmask: u32 = (1 << 10) - 1;

        let mut idx: usize = 0;
        let mut current_data: u32;
        let mut h_data: u32;
        let h_idx: u32 = 0;

        let mut real: isize;
        let mut imag: isize;

        h_data = csi_addr[idx] as u32;
        idx += 1;
        h_data += (csi_addr[idx] as u32) << 8;
        idx += 1;

        current_data = h_data & ((1 << 16) - 1);

        // loop for every subcarrier
        for k in 0..num_tones {
            // loop for each tx antenna
            for nc_idx in 0..nc {
                // loop for each rx antenna
                for nr_idx in 0..nr {
                    // if bits number is less than 10, then get next 16 bits?
                    if bits_left < 10 {
                        h_data = csi_addr[idx] as u32;
                        idx += 1;

                        h_data += (csi_addr[idx] as u32) << 8;
                        idx += 1;

                        current_data += h_data << bits_left;
                        bits_left += 16;
                    }

                    imag = (current_data & bitmask).try_into().unwrap();
                    imag = bit_convert(imag, 10);
                    // println!("> imag: {}, {:x}", imag, imag);

                    self.csi_matrix[nr_idx][nc_idx][k].im = imag;

                    bits_left -= 10;
                    current_data = current_data >> 10;


                    // if bits number is less than 10, then get next 16 bits?
                    if bits_left < 10 {
                        h_data = csi_addr[idx] as u32;
                        idx += 1;

                        h_data += (csi_addr[idx] as u32) << 8;
                        idx += 1;

                        current_data += h_data << bits_left;
                        bits_left += 16;
                    }

                    real = (current_data & bitmask).try_into().unwrap();
                    real = bit_convert(real,10);
                    self.csi_matrix[nr_idx][nc_idx][k].re = real;

                    bits_left -= 10;
                    current_data = current_data >> 10;
                }
            }
        }
    }
    
    // /// open_dev
    // pub fn open_dev(&mut self, path: &str) {
    //     self.file = fs::File::open(path)
    //         .expect(&format!("Cannot open {} device", path))
    // }

    // /// close_dev
    // pub fn close_dev(&mut self) {
    //     drop(&self.file)
    // }

    /// read_buf
    pub fn read_buf(&mut self, n: u64) -> usize {
        let fref = &mut self.file;
        let mut handle = fref.take(n);
        let actually = match handle.read(&mut self.buf) {
            Ok(x) => x,
            Err(_) => panic!("Failed to read")
        };

        actually
    }

    pub fn record_status(&mut self, cnt: usize) {
        if is_big_endian() {

            self.csi_status.tstamp =   
                  (((self.buf[0] as u64) << 56) & 0x00000000000000ff)
                | (((self.buf[1] as u64) << 48) & 0x000000000000ff00)
                | (((self.buf[2] as u64) << 40) & 0x0000000000ff0000)
                | (((self.buf[3] as u64) << 32) & 0x00000000ff000000)
                | (((self.buf[4] as u64) << 24) & 0x000000ff00000000)
                | (((self.buf[5] as u64) << 16) & 0x0000ff0000000000)
                | (((self.buf[6] as u64) <<  8) & 0x00ff000000000000)
                | (((self.buf[7] as u64)      ) & 0xff00000000000000);
            
            self.csi_status.csi_len =
                 (((self.buf[8] as u16) << 8) & 0xff00)
                | ((self.buf[9] as u16)       & 0x00ff);

            self.csi_status.channel =
                 (((self.buf[10] as u16) << 8) & 0xff00)
                | ((self.buf[11] as u16)       & 0x00ff);

            self.csi_status.buf_len =
                 (((self.buf[cnt-2] as u16) << 8) & 0xff00)
                | ((self.buf[cnt-1] as u16)       & 0x00ff);

            self.csi_status.payload_len =
                 (((self.buf[CSI_ST_LEN]   as u16) << 8) & 0xff00)
                | ((self.buf[CSI_ST_LEN+1] as u16)       & 0x00ff);

        } else {

            self.csi_status.tstamp =   
                  (((self.buf[7] as u64) << 56) & 0x00000000000000ff)
                | (((self.buf[6] as u64) << 48) & 0x000000000000ff00)
                | (((self.buf[5] as u64) << 40) & 0x0000000000ff0000)
                | (((self.buf[4] as u64) << 32) & 0x00000000ff000000)
                | (((self.buf[3] as u64) << 24) & 0x000000ff00000000)
                | (((self.buf[2] as u64) << 16) & 0x0000ff0000000000)
                | (((self.buf[1] as u64) <<  8) & 0x00ff000000000000)
                | (((self.buf[0] as u64)      ) & 0xff00000000000000);
            
            self.csi_status.csi_len =
                 (((self.buf[9] as u16) << 8) & 0xff00)
                | ((self.buf[8] as u16)       & 0x00ff);

            self.csi_status.channel =
                 (((self.buf[11] as u16) << 8) & 0xff00)
                | ((self.buf[10] as u16)       & 0x00ff);

            self.csi_status.buf_len =
                 (((self.buf[cnt-1] as u16) << 8) & 0xff00)
                | ((self.buf[cnt-2] as u16)       & 0x00ff);

            self.csi_status.payload_len =
                 (((self.buf[CSI_ST_LEN+1] as u16) << 8) & 0xff00)
                | ((self.buf[CSI_ST_LEN]   as u16)       & 0x00ff);
            
        }

        // common part

        self.csi_status.phyerr    = self.buf[12];
        self.csi_status.noise     = self.buf[13];
        self.csi_status.rate      = self.buf[14];
        self.csi_status.chanBW    = self.buf[15];
        self.csi_status.num_tones = self.buf[16];
        self.csi_status.nr        = self.buf[17];
        self.csi_status.nc        = self.buf[18];
        
        self.csi_status.rssi      = self.buf[19];
        self.csi_status.rssi_0    = self.buf[20];
        self.csi_status.rssi_1    = self.buf[21];
        self.csi_status.rssi_2    = self.buf[22];

    }


    /// record_csi_payload
    pub fn record_csi_payload(
        &mut self
    ) {
        let nr = self.csi_status.nr;
        let nc = self.csi_status.nc;
        let num_tones = self.csi_status.num_tones;
        let payload_len = self.csi_status.payload_len;
        let csi_len = self.csi_status.csi_len;

        for i in 1..=(payload_len as usize) {
            self.data_buf[i-1] = self.buf[CSI_ST_LEN + (csi_len as usize) + i + 1];
        }

        let csi_addr = self.buf[(CSI_ST_LEN+2)..].to_vec();
        self.fill_matrix(&csi_addr, nr.into(), nc.into(), num_tones.into());
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
