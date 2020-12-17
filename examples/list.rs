use std::net::{SocketAddr, ToSocketAddrs};

use anyhow::{format_err, Result};
use structopt::StructOpt;
use tokio::io::{AsyncBufReadExt, BufReader, ReadHalf};
use tokio::net::TcpStream;

use nntp::command::{AuthPart, Command};
use nntp::response::{parse_list, Capability, NewsgroupInfo, Response};

#[derive(StructOpt)]
struct Args {
    host_port: String,

    #[structopt(long)]
    user: Option<String>,

    #[structopt(long)]
    password: Option<String>,
}

struct UserPassword {
    user: String,
    password: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_args();
    let addr = args
        .host_port
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| format_err!("no usable IP addresses found for {}", args.host_port))?;

    let user_pass = match (args.user, args.password) {
        (Some(user), Some(password)) => Some(UserPassword { user, password }),
        (Some(_), None) => {
            return Err(format_err!("--user set without --password"));
        }
        (None, Some(_)) => {
            return Err(format_err!("--password set without --user"));
        }
        (None, None) => None,
    };

    connect(addr, user_pass).await?;

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
                        let newsgroups: Vec<NewsgroupInfo> = parse_list(&mut lines).await?;
                        println!("Newsgroups:");
                        for newsgroup in newsgroups.into_iter() {
                            println!("{}", newsgroup.name);
                        }
                    }
                    Response::CapabilitiesFollow { .. } => {
                        let _capabilities: Vec<Capability> = parse_list(&mut lines).await?;
                        //println!("{:?}", capabilities);
                    }
                    _ => {}
                }
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }

    Ok(())
}

async fn connect(addr: SocketAddr, user_pass: Option<UserPassword>) -> Result<()> {
    let stream = TcpStream::connect(addr).await?;
    let (reader, mut writer) = tokio::io::split(stream);

    Command::Capabilities.write_to_stream(&mut writer).await?;
    if let Some(user_pass) = user_pass {
        Command::AuthInfo(AuthPart::User(user_pass.user))
            .write_to_stream(&mut writer)
            .await?;
        Command::AuthInfo(AuthPart::Password(user_pass.password))
            .write_to_stream(&mut writer)
            .await?;
    }
    Command::List.write_to_stream(&mut writer).await?;
    Command::Quit.write_to_stream(&mut writer).await?;

    read_task(reader).await?;

    Ok(())
}
