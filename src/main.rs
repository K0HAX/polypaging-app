use clap::{Parser, ValueEnum};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use env_logger;
use log;
use polypaging;
use std::fs::File;
use std::io::BufReader;
use std::{io, process};

/*
mod poly;
use crate::poly::Packet;
*/

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Config::parse();

    let mut builder = env_logger::Builder::from_default_env();
    builder.filter_level(args.verbose.log_level_filter()).init();

    log::info!("Loading file: {}", args.filename);

    let f = File::open(&args.filename).unwrap_or_else(|err| {
        println!("Problem reading file: {err}");
        process::exit(1);
    });
    let mut file_handle = BufReader::new(f);

    let codec = match args.codec {
        Some(polypaging::rtpcodec::CodecFlag::G711u) => polypaging::rtpcodec::CodecFlag::G711u,
        Some(polypaging::rtpcodec::CodecFlag::G722) => polypaging::rtpcodec::CodecFlag::G722,
        None => {
            println!("Invalid codec!");
            process::exit(1);
        }
    };

    match args.mode {
        ModeSelect::Print => polypaging::do_print_stream(
            &mut file_handle,
            args.callerid_name,
            codec,
            args.channel_number,
        ),
        ModeSelect::Transmit => polypaging::do_transmit_stream(
            &mut file_handle,
            args.callerid_name,
            codec,
            args.channel_number,
        )
        .await
        .unwrap(),
    };

    Ok(())
}

// Config //
#[derive(Parser, Debug)]
#[command(author, version, verbatim_doc_comment)]
/// Poly Multicast Paging Transmitter
///
/// This program accepts audio in the form of either a
/// g711µ or g722 file, and sends it to multicast address
/// 224.0.1.116:5000 to page Poly phones.
///
/// Copyright © 2023-2024 Michael Englehorn
///
/// This program is free software: you can redistribute it and/or modify
/// it under the terms of the GNU General Public License as published by
/// the Free Software Foundation, either version 3 of the License, or
/// (at your option) any later version.
///
/// This program is distributed in the hope that it will be useful,
/// but WITHOUT ANY WARRANTY; without even the implied warranty of
/// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
/// GNU General Public License for more details.
///
/// You should have received a copy of the GNU General Public License
/// along with this program.  If not, see <https://www.gnu.org/licenses/>.
struct Config {
    /// Filename to send
    #[arg(short, long)]
    filename: String,

    /// Caller ID Name
    #[arg(short = 'n', long, default_value = "PC")]
    callerid_name: String,

    /// Codec Selection
    #[arg(short, long, value_enum, default_value = "g722")]
    codec: Option<polypaging::rtpcodec::CodecFlag>,

    /// Channel Number
    ///
    /// PTT = channels 1-25
    /// Paging = channels 26-50
    #[arg(short = 'u', long, default_value = "49")]
    channel_number: u8,

    /// Verbosity
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,

    /// Mode
    #[arg(short, long, value_enum, default_value = "transmit")]
    mode: ModeSelect,
}

#[derive(Clone, Debug, ValueEnum)]
enum ModeSelect {
    /// Print
    Print,

    /// Transmit
    Transmit,
}

// Vec<u8> //
struct PrintableU8Vec(Vec<u8>);

impl std::fmt::UpperHex for PrintableU8Vec {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut fmt_string = String::new();
        for byte in &self.0 {
            let byte_string: String = format!("{:02X}", byte);
            fmt_string.push_str(byte_string.as_str());
        }

        write!(f, "{}", fmt_string)
    }
}
