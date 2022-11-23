use ini::Ini;
use std::collections::HashMap;
use std::fmt::Debug;

fn parse_section(fhandler: &Ini, section: Option<&str>) -> Result<HashMap<String, String>, String> {
    let section_name = section.expect("Error while parsing a section name.");
    let section_params = fhandler
        .section(section)
        .ok_or(format!("Error reading section {}.", section_name))?;
    let mut section_data = HashMap::new();

    // Insert the section name as the name parameter in the hashmap
    section_data.insert("name".to_string(), section_name.to_string());

    // Insert each parameter and value in the hashmap
    section_params.iter().for_each(|(param_name, param_value)| {
        section_data.insert(param_name.to_string(), param_value.to_string());
    });
    Ok(section_data)
}

pub fn read_file<T>(filename: &str) -> Vec<T>
where
    T: TryFrom<HashMap<String, String>>,
    <T as TryFrom<HashMap<String, String>>>::Error: Debug,
{
    let mut vec = Vec::new();
    let fhandler = ini::Ini::load_from_file(filename)
        .expect(&format!("Error opening or parsing file {}.", filename));

    // Iterate in every section to get the needed
    fhandler.sections().for_each(|section| {
        let data = parse_section(&fhandler, section)
            .expect(&format!("Error while parsing file {}.", filename));

        let parsed_data =
            T::try_from(data).expect(&format!("Error while parsing file {}.", filename));
        vec.push(parsed_data);
    });
    vec
}

#[macro_export]
macro_rules! gen_matcher {
    (enum $e_name:ident { $( $field:ident),*, }) => {
        #[derive(Debug, Clone)]
        pub enum $e_name {
            $(
                $field,
            )*
        }

        impl std::str::FromStr for $e_name {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        stringify!($field) => Ok($e_name::$field),
                    )*
                    _ => Err(())
                }
            }
        }
    };
}

#[macro_export]
macro_rules! gen_readable_struct {
    (struct $s_name:ident { $( $field:ident:$type:ty),*, }) => {

        #[derive(Debug, Clone)]
        pub struct $s_name {
            pub $( $field: $type ),*
        }

        impl TryFrom<std::collections::HashMap<String, String>> for $s_name {
            type Error = String;
            fn try_from(value: std::collections::HashMap<String, String>) -> Result<Self, Self::Error> {
                let field_error = |field| format!("The field {} cannot be found.", field);
                let parse_error = |field, value| format!("The value {} of the field {} cannot be parsed.", value, field );

                $(
                    let $field = match value.get(stringify!($field)) {
                        None        => return Err(field_error(stringify!($field))),
                        Some(value) => match value.parse() {
                            Err(_)      => return Err(parse_error(stringify!($field), value)),
                            Ok(parsed)  => parsed,
                        }
                    };
                )*

                Ok(
                    $s_name {
                        $(
                            $field,
                        )*
                    }
                )
            }
        }
    };
}
