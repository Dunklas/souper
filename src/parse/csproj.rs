use std::{
    collections::{BTreeSet},
    io
};
use crate::soup::model::Soup;
use super::SoupSource;

pub struct CsProj {}

impl <R> SoupSource<R> for CsProj where R: io::Read {
    fn soups(reader: R) -> BTreeSet<Soup> {
        BTreeSet::new()
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
        "#.as_bytes();

        let soups = CsProj::soups(content);
        assert_eq!(1, soups.len());
        let expected_soup = Soup {
            name: "Azure.Messaging.ServiceBus".to_owned(),
            version: "7.2.1".to_owned(),
            meta: json!({})
        };
        assert_eq!(true, soups.contains(&expected_soup));
    }
}