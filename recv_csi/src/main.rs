use csi_types as csi;

const BUF_SIZE: u64 = 4096;

use std::time::Duration;
use crossbeam::channel::{bounded, tick, Receiver, select};

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "recv_csi", about = "Receive CSI data from /dev/CSI_dev")]
struct Opt {
    #[structopt(short, long)]
    debug: bool,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,

    #[structopt(long)]
    addr: String,
}

struct Processor {
    addr: String,
    client: reqwest::blocking::Client,
}

impl Processor {
    pub fn with_client(addr: String) -> Self {
        Self {
            addr: addr,
            client: reqwest::blocking::Client::new(),
        }
    }
    pub fn process_csi(&self, csi: &csi::CSI) {
        let data = bincode::serialize(&csi.to_ser())
            .unwrap();
            
        let res = self.client.post(&self.addr)
            .body(data)
            .send();
    }
}

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn main() -> Result<(), exitfailure::ExitFailure> {
    let opt = Opt::from_args();

    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_millis(0));

    let mut total_msg_cnt = 0;
    let mut csi = csi::CSI::with_file("/dev/CSI_dev");

    let processor = Processor::with_client(opt.addr);

    loop {
        select! {
            recv(ctrl_c_events) -> _ => {
                println!();
                println!("Ctrl-C received. Interrupting...");
                break;
            }
            recv(ticks) -> _ => {
                let have_read = csi.read_buf(BUF_SIZE);
                if have_read > 0 {
                    total_msg_cnt += 1;
                    csi.record_status(have_read);
                    csi.record_csi_payload();
                    println!("Received msg #{} | payload len: {}", total_msg_cnt, csi.csi_status.payload_len);
                    processor.process_csi(&csi);
                }
            }
        }
    }

    Ok(())
}
