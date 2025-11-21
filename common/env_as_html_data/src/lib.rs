extern crate html5ever;
extern crate markup5ever_rcdom as rcdom;

mod errors;
use errors::Error;
use html5ever::serialize::SerializeOpts;

use std::rc::Rc;
use std::str::FromStr;
use std::{collections::HashMap, hash::BuildHasher};

use html5ever::driver::ParseOpts;
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{ns, parse_document, serialize, Attribute, LocalName, QualName};
use rcdom::{Handle, Node, NodeData, RcDom, SerializableHandle};

pub(crate) fn env_as_html_data() {}

pub(crate) fn inject_data_into_html<S: BuildHasher>(
    data: &HashMap<String, String, S>,
    html: &str,
) -> Result<String, Error> {
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            ..Default::default()
        },
        ..Default::default()
    };
    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut html.as_bytes())
        .map_err(|e| Error::ParseError(e.to_string()))?;

    write_attrs_into_document(data, &dom.document)?;

    let document: SerializableHandle = dom.document.clone().into();

    let mut buf = Vec::new();
    serialize(&mut buf, &document, SerializeOpts::default())
        .map_err(|e| Error::SerializeError(e.to_string()))?;
    let output =
        std::str::from_utf8(buf.as_slice()).map_err(|e| Error::EncodeError(e.to_string()))?;
    Ok(output.to_string())
}

#[allow(clippy::type_complexity)]
fn write_attrs_into_document<S: BuildHasher>(
    data: &HashMap<String, String, S>,
    node: &Handle,
) -> Result<bool, Error> {
    // Closure around the reference-counted HTML DOM document/nodes, to support recursing to find the body element
    struct RecurseToMatch<'r> {
        f: &'r dyn Fn(&RecurseToMatch, &Rc<Node>) -> Result<bool, Error>,
    }
    let recurse_to_match = RecurseToMatch {
        f: &|recurse_to_match: &RecurseToMatch, n: &Rc<Node>| -> Result<bool, Error> {
            let m: Option<&Rc<Node>> = match n.data {
                NodeData::Element { ref name, .. } => {
                    if *name.local == *"body" {
                        Some(n)
                    } else {
                        None
                    }
                }

                _ => None,
            };

            if m.is_none() {
                let mut children = n.children.borrow_mut();
                for child in children.iter_mut() {
                    let mm = (recurse_to_match.f)(recurse_to_match, child);
                    if let Ok(true) = mm {
                        return Ok(true);
                    }
                }
                return Ok(false);
            }

            inject_public_attrs::<S>(data, m.expect("Document Node is already known to be Some"))?;
            Ok(true)
        },
    };

    (recurse_to_match.f)(&recurse_to_match, node)
}

fn inject_public_attrs<S: BuildHasher>(
    data: &HashMap<String, String, S>,
    body_element: &Rc<Node>,
) -> Result<(), Error> {
    let NodeData::Element {
        name: qual_name,
        attrs: body_attrs,
        ..
    } = &body_element.data
    else {
        return Err(Error::NoBodyElementError);
    };

    let mut keys: Vec<String> = data
        .keys()
        .filter(|k| k.starts_with("PUBLIC_") || k.starts_with("public_"))
        .cloned()
        .collect();
    keys.sort_unstable();

    for k in &keys {
        let new_attr = Attribute {
            name: QualName {
                prefix: qual_name.prefix.clone(),
                ns: ns!(),
                local: LocalName::from(format!("data-{}", k.to_lowercase())),
            },
            value: StrTendril::from_str(data[k].as_str())
                .expect("Data key is already known to exist"),
        };
        body_attrs.borrow_mut().push(new_attr);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::inject_data_into_html;
    use std::collections::HashMap;

    #[test]
    fn inject_data_into_html_succeeds() {
        let mut data: HashMap<String, String> = HashMap::new();
        data.insert(
            "PUBLIC_API_URL".to_string(),
            "https://api.example.com/v1".to_string(),
        );
        data.insert("PUBLIC_RELEASE_VERSION".to_string(), "v101".to_string());
        data.insert(
            "NOT_PUBLIC_VAR".to_string(),
            "non-public should not be included".to_string(),
        );
        let html =
            "<html><head><title>Hello World</title></head><body><h1>Hello World</h1></body></html>";
        let expected_html = r#"<html><head><title>Hello World</title></head><body data-public_api_url="https://api.example.com/v1" data-public_release_version="v101"><h1>Hello World</h1></body></html>"#;
        let result = inject_data_into_html(&data, html);
        assert!(result.is_ok());
        let result_value = result.unwrap();
        print!("{}", &result_value);
        assert_eq!(&result_value, expected_html);
    }
}
