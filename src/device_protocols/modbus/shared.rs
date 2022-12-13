use crate::models::device::WriteError;
use crate::models::tag::TagValue;
use crate::{gen_matcher, models::device::ReadError};
use tokio_modbus::client::Context;
use tokio_modbus::prelude::{Reader, Writer};

gen_matcher!(
    enum Swap {
        BigEndian,
        LittleEndian,
        BigEndianSwap,
        LittleEndianSwap,
    }
);

gen_matcher!(
    enum Type {
        Integer,
        Float,
    }
);

gen_matcher!(
    enum Command {
        Coil,
        Discrete,
        Holding,
        Input,
    }
);

pub async fn write(
    ctx: &mut Context,
    command: &Command,
    address: u16,
    value_to_write: &[u16],
) -> Result<(), WriteError>
where
    Context: Writer,
{
    match command {
        Command::Coil => {
            ctx.write_single_coil(address, from_byte_slice_to_coil(&value_to_write))
                .await
        }
        Command::Discrete => unimplemented!("A discrete register cannot be written."),
        Command::Holding => ctx.write_multiple_registers(address, &value_to_write).await,
        Command::Input => unimplemented!("An input register cannot be written."),
    }
    .map_err(|err| WriteError(err.to_string()))?;

    ctx.disconnect()
        .await
        .map_err(|err| WriteError(err.to_string()))?;
    Ok(())
}

pub async fn read(
    ctx: &mut Context,
    command: &Command,
    address: u16,
    length: u16,
) -> Result<Vec<u16>, ReadError>
where
    Context: Reader,
{
    let readed_data = match command {
        Command::Coil => from_coil_to_word(ctx.read_coils(address, length)),
        Command::Discrete => from_coil_to_word(ctx.read_discrete_inputs(address, length)),
        Command::Holding => ctx.read_holding_registers(address, length),
        Command::Input => ctx.read_input_registers(address, length),
    };

    let readed_data = readed_data
        .await
        .map_err(|err| ReadError(err.to_string()))?;

    ctx.disconnect()
        .await
        .map_err(|err| ReadError(err.to_string()))?;
    Ok(readed_data)
}

use core::future::Future;
use core::pin::Pin;
fn from_coil_to_word<'a>(
    data: impl Future<Output = Result<Vec<bool>, std::io::Error>> + std::marker::Send + 'a,
) -> Pin<Box<dyn Future<Output = Result<Vec<u16>, std::io::Error>> + std::marker::Send + 'a>> {
    Box::pin(async {
        Ok(data
            .await?
            .iter()
            .map(|b| match b {
                true => 1,
                false => 0,
            })
            .collect())
    })
}

pub fn parse_write(data: &TagValue, swap: &Swap) -> Vec<u16> {
    let data = match data {
        TagValue::F32(val) => val.to_be_bytes(),
        TagValue::I32(val) => val.to_be_bytes(),
    };
    let data = data.map(|b| b as u16).to_vec();

    match swap {
        Swap::LittleEndian => data.iter().map(swap_bytes).rev().collect(),
        Swap::BigEndian => data,
        Swap::LittleEndianSwap => swap_words(data.iter().map(swap_bytes).rev().collect()),
        Swap::BigEndianSwap => swap_words(data),
    }
}

pub fn parse_readed(data: Vec<u16>, swap: &Swap, data_type: &Type, multiplier: &f32) -> TagValue {
    let data: Vec<u16> = match swap {
        Swap::LittleEndian => data.iter().map(swap_bytes).rev().collect(),
        Swap::BigEndian => data,
        Swap::LittleEndianSwap => swap_words(data.iter().map(swap_bytes).rev().collect()),
        Swap::BigEndianSwap => swap_words(data),
    };

    let data_as_string = match data_type {
        Type::Integer => data
            .iter()
            .fold(0i32, |acc, &num| acc << 16 | num as i32)
            .to_string(),
        Type::Float => {
            let num = data.iter().fold(0u32, |acc, &num| acc << 16 | num as u32);
            format!("{:.2}", f32::from_bits(num))
        }
    };

    let readed_value: f32 = data_as_string.parse().unwrap();
    let scaled_value = readed_value * multiplier;

    match is_integer(scaled_value) {
        true => TagValue::I32(scaled_value as i32),
        false => TagValue::F32(scaled_value),
    }
}

pub fn from_byte_slice_to_coil(bytes: &[u16]) -> bool {
    bytes.iter().sum::<u16>() != 0
}

fn is_integer(value: f32) -> bool {
    value == value.round()
}

fn swap_words(words: Vec<u16>) -> Vec<u16> {
    let mut data = words;
    data.swap(0, 1);
    data.to_vec()
}

fn swap_bytes(word: &u16) -> u16 {
    word.rotate_left(8)
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_from_coil_to_word() {
        use super::from_coil_to_word;

        let async_response = |bool_vec: Vec<bool>| async move { Ok(bool_vec) };
        assert_eq!(
            vec![1, 1, 0, 0],
            from_coil_to_word(async_response(vec![true, true, false, false]))
                .await
                .unwrap()
        );
        assert_eq!(
            vec![1, 0, 0, 1],
            from_coil_to_word(async_response(vec![true, false, false, true]))
                .await
                .unwrap()
        );
        assert_ne!(
            vec![0, 0, 0, 0],
            from_coil_to_word(async_response(vec![true, true, false, false]))
                .await
                .unwrap()
        );
    }

    #[test]
    fn test_parse_readed() {
        use super::{parse_readed, Swap, TagValue, Type};

        let u16_vec: Vec<u16> = vec![0, 0xE8];
        assert_eq!(
            TagValue::F32(23.2),
            parse_readed(u16_vec.to_owned(), &Swap::BigEndian, &Type::Integer, &0.1)
        );
        assert_eq!(
            TagValue::I32(232),
            parse_readed(u16_vec.to_owned(), &Swap::BigEndian, &Type::Integer, &1.0)
        );
        assert_eq!(
            TagValue::I32(15204352),
            parse_readed(
                u16_vec.to_owned(),
                &Swap::BigEndianSwap,
                &Type::Integer,
                &1.0
            )
        );
        assert_eq!(
            TagValue::I32(15204352),
            parse_readed(
                u16_vec.to_owned(),
                &Swap::BigEndianSwap,
                &Type::Integer,
                &1.0
            )
        );
        assert_eq!(
            TagValue::I32(-402653184),
            parse_readed(
                u16_vec.to_owned(),
                &Swap::LittleEndian,
                &Type::Integer,
                &1.0
            )
        );
        assert_eq!(
            TagValue::I32(59392),
            parse_readed(
                u16_vec.to_owned(),
                &Swap::LittleEndianSwap,
                &Type::Integer,
                &1.0
            )
        );
    }
}
