use anyhow::{bail, Error};
use bytes::BufMut;
use std::iter::once;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum PipeAction {
    Error = 0x0,
    Request = 0x1,
    Response = 0x2,
    Part = 0x3,
    Close = 0x4,
}

impl TryFrom<u8> for PipeAction {
    type Error = Error;

    fn try_from(num: u8) -> Result<Self, <PipeAction as TryFrom<u8>>::Error> {
        match num {
            0x0 => Ok(PipeAction::Error),
            0x1 => Ok(PipeAction::Request),
            0x2 => Ok(PipeAction::Response),
            0x3 => Ok(PipeAction::Part),
            0x4 => Ok(PipeAction::Close),
            _ => bail!("invalid pipeaction"),
        }
    }
}

/// A request made on a pipe,
pub struct PipeRequest {
    pub channel_id: u16,
    pub service_id: u16,
    pub call_id: u16,
    pub data: Vec<u8>,
}

impl PipeRequest {
    /// Get a slice of the data that containts the actual request payload without the pipe information
    pub fn payload_slice(&self) -> &[u8] {
        &self.data[7..]
    }
}

/// A message comming into a Pipe
pub enum PipeMessage {
    Request(PipeRequest),
    Close,
}

impl TryFrom<Vec<u8>> for PipeMessage {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // Pipe messages have to be atleast 3 bytes long
        // A byte for the action and two bytes for the channel
        if value.len() < 3 {
            bail!("invalid pipe message");
        }

        match PipeAction::try_from(value[0])? {
            PipeAction::Close => Ok(PipeMessage::Close),

            PipeAction::Request => {
                // A request must have the pipe prefixes which is 3 bytes
                // and two bytes for service id and two bytes for call id
                if value.len() < 7 {
                    bail!("invalid pipe request");
                }

                let channel_id = u16::from_be_bytes([value[1], value[2]]);
                let service_id = u16::from_be_bytes([value[3], value[4]]);
                let call_id = u16::from_be_bytes([value[5], value[6]]);

                Ok(PipeMessage::Request(PipeRequest {
                    channel_id,
                    service_id,
                    call_id,
                    data: value,
                }))
            }

            _ => bail!("invalid pipe action from the client"),
        }
    }
}

pub fn error_message(channel: u16, message: &str) -> Vec<u8> {
    once(PipeAction::Error as u8)
        .chain(channel.to_be_bytes())
        .chain(message.bytes())
        .collect::<Vec<_>>()
}

pub fn part_message(channel: u16, message: impl prost::Message) -> anyhow::Result<Vec<u8>>  {
    let mut out = Vec::with_capacity(3 + message.encoded_len());

    out.push(PipeAction::Part as u8);
    out.put_u16(channel);

    message.encode(&mut out)?;

    Ok(out)
}

pub fn response_message(channel: u16, message: impl prost::Message) -> anyhow::Result<Vec<u8>>  {
    let mut out = Vec::with_capacity(3 + message.encoded_len());

    out.push(PipeAction::Response as u8);
    out.put_u16(channel);

    message.encode(&mut out)?;

    Ok(out)
}
