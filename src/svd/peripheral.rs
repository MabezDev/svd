#[cfg(feature = "unproven")]
use std::collections::HashMap;

use xmltree::Element;

use crate::elementext::ElementExt;
use crate::parse;

#[cfg(feature = "unproven")]
use crate::encode::{Encode, EncodeChildren};
#[cfg(feature = "unproven")]
use crate::new_element;
use crate::types::Parse;

use crate::error::*;
use crate::svd::{
    addressblock::AddressBlock, interrupt::Interrupt, registercluster::RegisterCluster,
    registerproperties::RegisterProperties,
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, derive_builder::Builder)]
pub struct Peripheral {
    /// The string identifies the peripheral. Peripheral names are required to be unique for a device
    pub name: String,

    /// The string specifies the version of this peripheral description
    #[builder(default)]
    pub version: Option<String>,

    #[builder(default)]
    pub display_name: Option<String>,

    #[builder(default)]
    pub group_name: Option<String>,

    /// The string provides an overview of the purpose and functionality of the peripheral
    #[builder(default)]
    pub description: Option<String>,

    /// Lowest address reserved or used by the peripheral
    pub base_address: u32,

    /// Specify an address range uniquely mapped to this peripheral
    #[builder(default)]
    pub address_block: Option<AddressBlock>,

    /// A peripheral can have multiple associated interrupts
    #[builder(default)]
    pub interrupt: Vec<Interrupt>,

    #[builder(default)]
    pub default_register_properties: RegisterProperties,

    /// Group to enclose register definitions.
    /// `None` indicates that the `<registers>` node is not present
    #[builder(default)]
    pub registers: Option<Vec<RegisterCluster>>,

    /// Specify the peripheral name from which to inherit data. Elements specified subsequently override inherited values
    #[builder(default)]
    pub derived_from: Option<String>,

    // Reserve the right to add more fields to this struct
    #[builder(default)]
    _extensible: (),
}

impl Parse for Peripheral {
    type Object = Peripheral;
    type Error = anyhow::Error;

    fn parse(tree: &Element) -> Result<Peripheral> {
        if tree.name != "peripheral" {
            return Err(ParseError::NotExpectedTag(tree.clone(), "peripheral".to_string()).into());
        }
        let name = tree.get_child_text("name")?;
        Peripheral::_parse(tree, name.clone()).with_context(|| format!("In peripheral `{}`", name))
    }
}

impl Peripheral {
    fn _parse(tree: &Element, name: String) -> Result<Peripheral> {
        PeripheralBuilder::default()
            .name(name)
            .version(tree.get_child_text_opt("version")?)
            .display_name(tree.get_child_text_opt("displayName")?)
            .group_name(tree.get_child_text_opt("groupName")?)
            .description(tree.get_child_text_opt("description")?)
            .base_address(tree.get_child_u32("baseAddress")?)
            .address_block(parse::optional::<AddressBlock>("addressBlock", tree)?)
            .interrupt({
                let interrupt: Result<Vec<_>, _> = tree
                    .children
                    .iter()
                    .filter(|t| t.name == "interrupt")
                    .enumerate()
                    .map(|(e, i)| Interrupt::parse(i).with_context(|| format!("Parsing interrupt #{}", e)))
                    .collect();
                interrupt?
            })
            .default_register_properties(RegisterProperties::parse(tree)?)
            .registers(if let Some(registers) = tree.get_child("registers") {
                let rs: Result<Vec<_>, _> = registers
                    .children
                    .iter()
                    .map(RegisterCluster::parse)
                    .collect();
                Some(rs?)
            } else {
                None
            })
            .derived_from(tree.attributes.get("derivedFrom").map(|s| s.to_owned()))
            .build()
            .map_err(|e| anyhow::anyhow!(e))
    }
}

#[cfg(feature = "unproven")]
impl Encode for Peripheral {
    type Error = anyhow::Error;

    fn encode(&self) -> Result<Element> {
        let mut elem = Element {
            prefix: None,
            namespace: None,
            namespaces: None,
            name: String::from("peripheral"),
            attributes: HashMap::new(),
            children: vec![new_element("name", Some(self.name.clone()))],
            text: None,
        };

        if let Some(v) = &self.version {
            elem.children
                .push(new_element("version", Some(format!("{}", v))));
        };
        if let Some(v) = &self.display_name {
            elem.children
                .push(new_element("displayName", Some(format!("{}", v))));
        };
        if let Some(v) = &self.group_name {
            elem.children
                .push(new_element("groupName", Some(format!("{}", v))));
        };
        if let Some(v) = &self.description {
            elem.children
                .push(new_element("description", Some(format!("{}", v))));
        };
        elem.children.push(new_element(
            "baseAddress",
            Some(format!("0x{:.08x}", self.base_address)),
        ));

        elem.children
            .extend(self.default_register_properties.encode()?);

        if let Some(v) = &self.address_block {
            elem.children.push(v.encode()?);
        };

        let interrupts: Result<Vec<_>, _> = self.interrupt.iter().map(Interrupt::encode).collect();

        elem.children.append(&mut interrupts?);

        if let Some(v) = &self.registers {
            let children: Result<Vec<_>, _> = v.iter().map(|e| e.encode()).collect();

            elem.children.push(Element {
                prefix: None,
                namespace: None,
                namespaces: None,
                name: String::from("registers"),
                attributes: HashMap::new(),
                children: children?,
                text: None,
            });
        };

        if let Some(v) = &self.derived_from {
            elem.attributes
                .insert(String::from("derivedFrom"), format!("{}", v));
        }

        Ok(elem)
    }
}

// TODO: add Peripheral encode / decode tests
