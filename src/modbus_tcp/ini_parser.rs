use super::modbus::{ReadCommand, ModbusTcpTagReadRequest, ModbusTcpConnectionParameters, Swap, Type};
use ini::Ini;

pub const NOT_TAGS_SECTIONS: [&str; 1] = ["CONNECTION_PARAMETERS"];
pub struct IniFile(pub String);

impl IniFile {
    pub fn get_ini_data(&self) -> (ModbusTcpConnectionParameters, Vec<ModbusTcpTagReadRequest>) {
        let ini = Ini::load_from_file(&self.0)
            .expect(format!("Invalid {} file or not found.", self.0).as_str());

        (
            self.get_connection_parameters(&ini),
            self.get_csv_structure(&ini),
        )
    }

    fn get_connection_parameters(&self, ini: &Ini) -> ModbusTcpConnectionParameters {
        let section_data = ini
            .section(Some(NOT_TAGS_SECTIONS[0]))
            .expect(format!("Config file {} cannot find or parse the field ip.", self.0).as_str());

        let field = "ip";
        let ip_address = section_data
            .get(field)
            .expect(format!("Config file {} cannot find the field {}.", self.0, field).as_str())
            .to_string();

        let field = "port";
        let port = section_data
            .get("port")
            .expect(format!("Config file {} cannot find the field {}.", self.0, field).as_str())
            .parse()
            .expect(format!("Config file {} cannot parse the field {}.", self.0, field).as_str());

        let field = "slave";
        let slave = section_data
            .get("slave")
            .expect(format!("Config file {} cannot find the field {}.", self.0, field).as_str())
            .parse()
            .expect(format!("Config file {} cannot parse the field {}.", self.0, field).as_str());

        ModbusTcpConnectionParameters {
            ip_address,
            port,
            slave,
        }
    }

    fn get_csv_structure(&self, ini: &Ini) -> Vec<ModbusTcpTagReadRequest> {
        let sections = ini
            .sections()
            .map(|s| {
                s.expect(format!("There is an issue parsing sections on file {}.", self.0).as_str())
            })
            .filter(|&s| !NOT_TAGS_SECTIONS.contains(&s));

        sections
            .map(|section| {
                let data = ini.section(Some(section)).unwrap();
                let unwrap = |field: &str| {
                    format!(
                        "Missing field {} in section {} on file {}.",
                        field, section, self.0
                    )
                };

                let (name, command, address, length, swap, data_type) = (
                    section.to_string(),
                    parse_command(data.get("command").expect(&unwrap("command"))),
                    parse_integer(data.get("address").expect(&unwrap("address"))),
                    parse_integer(data.get("length").expect(&unwrap("length"))),
                    parse_swap(data.get("swap").expect(&unwrap("swap"))),
                    parser_type(data.get("data_type").expect(&unwrap("data_type"))),
                );

                if length > 2 {
                    panic!(
                        "The length {} in section {} on file {} cannot be greater than two.",
                        length, section, self.0
                    );
                }

                ModbusTcpTagReadRequest {
                    name,
                    command,
                    address,
                    length,
                    swap,
                    data_type,
                }
            })
            .collect()
    }
}

fn parse_integer(integer: &str) -> u16 {
    integer
        .parse()
        .expect(format!("{} cannot be parsed as a number.", integer).as_str())
}

fn parse_command(command: &str) -> ReadCommand {
    match command {
        "ReadCoil" => ReadCommand::ReadCoil,
        "ReadDiscrete" => ReadCommand::ReadDiscrete,
        "ReadHolding" => ReadCommand::ReadHolding,
        "ReadInput" => ReadCommand::ReadInput,
        x => unimplemented!("Not valid command {}", x),
    }
}

fn parse_swap(swap: &str) -> Swap {
    match swap {
        "BigEndian" => Swap::BigEndian,
        "LittleEndian" => Swap::LittleEndian,
        "BigEndianSwap" => Swap::BigEndianSwap,
        "LittleEndianSwap" => Swap::LittleEndianSwap,
        x => unimplemented!("Not valid swap {}", x),
    }
}

fn parser_type(data_type: &str) -> Type {
    match data_type {
        "Integer" => Type::Integer,
        "Float" => Type::Float,
        x => unimplemented!("Not valid data type {}", x),
    }
}
