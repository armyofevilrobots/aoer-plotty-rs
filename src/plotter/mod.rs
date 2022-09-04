use serialport;
use std::io::Write;
use std::io::{BufRead, BufReader, BufWriter};
use std::ops::DerefMut;
use std::time::Duration;

pub mod error;
pub use error::*;

const DEFAULT_TIMEOUT: u64 = 30000;

pub trait PlotterTransport {
    fn write_line(&mut self, buf: &str) -> std::io::Result<()>;
    fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize>;
    fn flush(&mut self) -> std::io::Result<()>;
}

pub enum PlotterConnection {
    SerialReadWrite(Box<dyn BufRead>, Box<dyn Write>),
}

impl PlotterConnection {
    /// Given a URI in the form of serial:///dev/ttySomethingOrOther@115200,
    /// open up a serial connection on the /dev/ttySomethingOrOther at 115200 bps.
    pub fn from_uri(uri: &str) -> Result<PlotterConnection, PlotterConnectionError> {
        let url = url::Url::parse(uri)?;
        if url.scheme() == "serial" {
            let parts: Vec<&str> = url.path().split("@").collect();
            if parts.len() == 2 {
                let path = parts[0].to_string();
                let bps = parts[1].to_string().parse::<u32>()?;
                let sp = serialport::new(path, bps)
                    .timeout(Duration::from_millis(DEFAULT_TIMEOUT))
                    .open()?;
                let reader = BufReader::new(sp.try_clone()?);
                let writer = BufWriter::new(sp);
                Ok(PlotterConnection::SerialReadWrite(
                    Box::new(reader),
                    Box::new(writer),
                ))
            } else {
                Err(PlotterConnectionError::UnknownError)
            }
        } else {
            Err(PlotterConnectionError::UnknownError)
        }
    }
}

impl PlotterTransport for PlotterConnection {
    fn write_line(&mut self, buf: &str) -> std::io::Result<()> {
        match self {
            PlotterConnection::SerialReadWrite(_, ref mut bwrite) => bwrite
                .deref_mut()
                .write_all((buf.to_owned() + "\n").as_bytes()),
        }
    }

    fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize> {
        match self {
            PlotterConnection::SerialReadWrite(ref mut bread, _) => {
                bread.deref_mut().read_line(buf)
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            PlotterConnection::SerialReadWrite(_, ref mut bwrite) => bwrite.deref_mut().flush(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    /*
    #[tests]
    fn test_from_url(){
        let mut pc=PlotterConnection::from_uri("serial:///dev/foobar@38400").expect("This sucks");
        &pc.write_line("FAILED");
        let flushed = &pc.flush();
        println!("Flush result: {:?}", flushed);
        match flushed {
            Ok(_) => assert!(false), // We failed the tests
            Err(err) => println!("The failure: {}", err.to_string())
        }

    }
    */

    #[test]
    fn test_trial() {
        // let mut sp = serialport::new("/dev/tty.usbmodem14122301", 115200)
        //     .timeout(Duration::from_millis(5000))
        //     .open()
        //     .expect("Failed to open");
        // let mut reader = BufReader::new(sp.try_clone().expect("Failed to clone port"));
        // let mut writer = BufWriter::new(sp);
        // println!("Connected.");
        // let mut pc = PlotterConnection::SerialReadWrite(Box::new(reader), Box::new(writer));
        let mut pc = PlotterConnection::from_uri("serial:///dev/tty.usbmodem14122301@115200")
            .expect("Failed!");

        //&pc.write_line("M999");
        &pc.write_line("M280 S8");
        &pc.write_line("G28 X Y");
        &pc.write_line("G90");
        &pc.write_line("G92 X0 Y0");
        &pc.write_line("G4 P150");
        &pc.write_line("M281");

        // &pc.write_line("G0 F100");
        // &pc.write_line("G1 F100");
        // &pc.write_line("M281");
        // &pc.write_line("G92 X0 Y0");
        // &pc.write_line("G90");
        // &pc.write_line("G92 X0 Y0");

        &pc.flush();
        let mut buf = String::new();
        for i in 0..6 {
            &pc.read_line(&mut buf).expect("Failed to read...");
            println!("{}> '{}'", i, buf.trim());
            buf.clear();
        }
        // loop{
        //     &pc.read_line(&mut buf).expect("Overage");
        //     println!("Extra line: '{}'", buf.trim());
        //     buf.clear();
        // }
        /*        println!("Read: {} bytes",pc.read_line(&mut buf).expect("Failed to read..."));
                println!("1: {}", buf.trim());
                assert_eq!(buf.trim(), "ok");
                println!("Read {} bytes", pc.read_line(&mut buf).expect("Failed to read..."));
                println!("2: {}", buf.trim());
                assert_eq!(buf.trim(), "ok");
        */
    }
}
