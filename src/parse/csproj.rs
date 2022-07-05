use std::{
    collections::{BTreeSet, HashMap},
    io
};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::json;
use super::SoupSource;
use crate::soup::model::{Soup, SoupSourceParseError};

pub struct CsProj {}

impl<R> SoupSource<R> for CsProj
where
    R: io::BufRead,
{
    fn soups(reader: R) -> Result<BTreeSet<Soup>, SoupSourceParseError> {
        let mut reader = Reader::from_reader(reader);
        reader.trim_text(true);
        reader.expand_empty_elements(true);

        let mut soups: BTreeSet<Soup> = BTreeSet::new();
        let mut buf = Vec::new();
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => if let b"PackageReference" = e.name() {
                    let attributes_by_key = e.attributes()
                        .filter_map(|attribute| attribute.ok())
                        .map(|attribute| (attribute.key.to_vec(), attribute.value.to_vec()))
                        .collect::<HashMap<Vec<u8>, Vec<u8>>>();
                    let name = attribute_value(&attributes_by_key, "Include")?;
                    let version = attribute_value(&attributes_by_key, "Version")?;
                        soups.insert(Soup {
                            name,
                            version,
                            meta: json!({})
                        });
                },
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(SoupSourceParseError{
                        message: format!("Invalid XML structure {}", e)
                    });
                }
                _ => {}
            }
        }
        buf.clear();
        Ok(soups)
    }
}

fn attribute_value(attributes: &HashMap<Vec<u8>, Vec<u8>>, key: &str) -> Result<String, SoupSourceParseError> {
    match attributes.get(key.as_bytes()) {
        Some(value) => match String::from_utf8(value.clone()) {
            Ok(value) => Ok(value),
            Err(_e) => {
                return Err(SoupSourceParseError{
                    message: format!("Unable to parse attribute {} as utf8", key)
                });
            }
        },
        None => {
            return Err(SoupSourceParseError{
                message: format!("Missing required attribute: {}", key)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn single_dependency() {
        let content: &[u8] = r#"
<Project Sdk="Microsoft.NET.Sdk.Web">
    <ItemGroup>
        <PackageReference Include="Azure.Messaging.ServiceBus" Version="7.2.1" />
    </ItemGroup>
</Project>
        "#
        .as_bytes();

        let result = CsProj::soups(content);
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(1, soups.len());
        let expected_soup = Soup {
            name: "Azure.Messaging.ServiceBus".to_owned(),
            version: "7.2.1".to_owned(),
            meta: json!({}),
        };
        assert_eq!(true, soups.contains(&expected_soup));
    }

    #[test]
    fn multiple_dependencies() {
        let content: &[u8] = r#"
<Project Sdk="Microsoft.NET.Sdk.Web">
    <ItemGroup>
        <PackageReference Include="Azure.Messaging.ServiceBus" Version="7.2.1" />
        <PackageReference Include="Swashbuckle.AspNetCore" Version="6.3.1" />
    </ItemGroup>
</Project>
        "#
        .as_bytes();

        let result = CsProj::soups(content);
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(2, soups.len());
        let expected_soups = vec![
            Soup { name: "Azure.Messaging.ServiceBus".to_owned(), version: "7.2.1".to_owned(), meta: json!({}) },
            Soup { name: "Swashbuckle.AspNetCore".to_owned(), version: "6.3.1".to_owned(), meta: json!({}) }
        ].into_iter().collect::<BTreeSet<Soup>>();
        assert_eq!(expected_soups, soups);
    }

    #[test]
    fn no_dependencies() {
        let content = r#"
<Project Sdk="Microsoft.NET.Sdk.Web">
    <ItemGroup>
    </ItemGroup>
</Project>
        "#.as_bytes();

        let result = CsProj::soups(content);
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(0, soups.len());
    }
}
