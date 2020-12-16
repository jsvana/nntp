use std::net::{SocketAddr, ToSocketAddrs};

use anyhow::{format_err, Result};
use structopt::StructOpt;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf};
use tokio::net::TcpStream;

use nntp::response::{parse_newsgroup_list, Response};

#[derive(StructOpt)]
struct Args {
    host_port: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();
    let addr = args.host_port.to_socket_addrs()?.next().ok_or_else(|| format_err!("no usable IP addresses found for {}", args.host_port))?;

    connect(&addr).await?;

    Ok(())
}

/*
async fn write_task(reader: ReadHalf<TcpStream>) -> Result<()> {
    let server_reader = BufReader::new(reader);
    let mut lines = server_reader.lines();
    while let Some(line) = lines.next_line().await? {
    }

    Ok(())
}
*/

async fn read_task(reader: ReadHalf<TcpStream>) -> Result<()> {
    let server_reader = BufReader::new(reader);
    let mut lines = server_reader.lines();
    while let Some(line) = lines.next_line().await? {
        match line.parse::<Response>() {
            Ok(response) => {
                println!("{:?}", response);
                match response {
                    Response::InformationFollows { .. } => {
                        let newsgroup_list = parse_newsgroup_list(&mut lines).await?;
                        println!("{:?}", newsgroup_list);
                    },
                    _ => {
                    }
                }
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }

    Ok(())
}

async fn connect(
    addr: &SocketAddr,
) -> Result<()> {
    let stream = TcpStream::connect(addr).await?;
    let (reader, mut writer) = tokio::io::split(stream);

    writer.write_all(b"LIST\r\n").await?;
    writer.write_all(b"QUIT\r\n").await?;

    read_task(reader).await?;

    Ok(())
}