use std::fmt;

use anyhow::Result;
use async_trait::async_trait;
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;

pub enum AuthPart {
    User(String),
    Password(String),
}

pub enum Command {
    /*
       +-------------------+-----------------------+---------------+
       | Command           | Indicating capability | Definition    |
       +-------------------+-----------------------+---------------+
       | ARTICLE           | READER                | Section 6.2.1 |
       | BODY              | READER                | Section 6.2.3 |
       | CAPABILITIES      | mandatory             | Section 5.2   |
       | DATE              | READER                | Section 7.1   |
       | GROUP             | READER                | Section 6.1.1 |
       | HDR               | HDR                   | Section 8.5   |
       | HEAD              | mandatory             | Section 6.2.2 |
       | HELP              | mandatory             | Section 7.2   |
       | IHAVE             | IHAVE                 | Section 6.3.2 |
       | LAST              | READER                | Section 6.1.3 |
       | LIST              | LIST                  | Section 7.6.1 |
       | LIST ACTIVE.TIMES | LIST                  | Section 7.6.4 |
       | LIST ACTIVE       | LIST                  | Section 7.6.3 |
       | LIST DISTRIB.PATS | LIST                  | Section 7.6.5 |
       | LIST HEADERS      | HDR                   | Section 8.6   |
       | LIST NEWSGROUPS   | LIST                  | Section 7.6.6 |
       | LIST OVERVIEW.FMT | OVER                  | Section 8.4   |
       | LISTGROUP         | READER                | Section 6.1.2 |
       | MODE READER       | MODE-READER           | Section 5.3   |
       | NEWGROUPS         | READER                | Section 7.3   |
       | NEWNEWS           | NEWNEWS               | Section 7.4   |
       | NEXT              | READER                | Section 6.1.4 |
       | OVER              | OVER                  | Section 8.3   |
       | POST              | POST                  | Section 6.3.1 |
       | QUIT              | mandatory             | Section 5.4   |
       | STAT              | mandatory             | Section 6.2.4 |
       | CAPABILITIES      | mandatory             | Section 5.2   |
       | HEAD              | mandatory             | Section 6.2.2 |
       | HELP              | mandatory             | Section 7.2   |
       | QUIT              | mandatory             | Section 5.4   |
       | STAT              | mandatory             | Section 6.2.4 |
       +-------------------+-----------------------+---------------+
    */
    Capabilities,
    List,
    Quit,

    Group(String),

    AuthInfo(AuthPart),
}

#[async_trait]
pub trait WriteCommand {
    async fn write_command(&mut self, command: Command) -> Result<()>;
}

#[async_trait]
impl WriteCommand for WriteHalf<TcpStream> {
    async fn write_command(&mut self, command: Command) -> Result<()> {
        Ok(self
            .write_all(format!("{}\r\n", command).as_bytes())
            .await?)
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Capabilities => write!(f, "CAPABILITIES"),
            Command::List => write!(f, "LIST"),
            Command::Quit => write!(f, "QUIT"),

            Command::Group(name) => write!(f, "GROUP {}", name),

            Command::AuthInfo(auth_part) => {
                write!(f, "AUTHINFO ")?;
                match auth_part {
                    AuthPart::User(user) => write!(f, "USER {}", user),
                    AuthPart::Password(password) => write!(f, "PASS {}", password),
                }
            }
        }
    }
}
