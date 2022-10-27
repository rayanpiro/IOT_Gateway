gen_readable_struct!(
    struct ModbusRtuConnection {
        name: String,
        baudrate: u32,
        parity: u32,
        odd: bool,
        slave: u32,
    }
);

gen_readable_struct!(
    struct ModbusTcpTag {
        name: String,
        address: u16,
        length: u16,
        command: Command,
        swap: Swap,
        data_type: Type,
    }
);