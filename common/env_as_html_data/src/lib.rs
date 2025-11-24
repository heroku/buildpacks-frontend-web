extern crate html5ever;
extern crate markup5ever_rcdom as rcdom;

mod errors;
use errors::Error;

use std::{collections::HashMap, hash::BuildHasher, rc::Rc, str::FromStr};

use html5ever::driver::ParseOpts;
use html5ever::serialize::SerializeOpts;
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{ns, parse_document, serialize, Attribute, LocalName, QualName};
use rcdom::{Handle, Node, NodeData, RcDom, SerializableHandle};

pub(crate) fn env_as_html_data() {}

pub(crate) fn parse_html_and_inject_data<S: BuildHasher>(
    data: &HashMap<String, String, S>,
    html: &str,
) -> Result<(bool, String), Error> {
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

    if let (_, false) = match_html_body_and_inject_data(data, &dom.document)? {
        return Ok((false, html.to_owned()));
    };

    let document: SerializableHandle = dom.document.clone().into();

    let mut buf = Vec::new();
    serialize(&mut buf, &document, SerializeOpts::default())
        .map_err(|e| Error::SerializeError(e.to_string()))?;
    let output =
        std::str::from_utf8(buf.as_slice()).map_err(|e| Error::EncodeError(e.to_string()))?;
    Ok((true, output.to_string()))
}

#[allow(clippy::type_complexity)]
fn match_html_body_and_inject_data<S: BuildHasher>(
    data: &HashMap<String, String, S>,
    node: &Handle,
) -> Result<(bool, bool), Error> {
    // Closure around the reference-counted HTML DOM document/nodes, to support recursing to find the body element
    struct RecurseToMatch<'r> {
        f: &'r dyn Fn(&RecurseToMatch, &Rc<Node>) -> Result<(bool, bool), Error>,
    }
    let recurse_to_match = RecurseToMatch {
        f: &|recurse_to_match: &RecurseToMatch, n: &Rc<Node>| -> Result<(bool, bool), Error> {
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
                    match (recurse_to_match.f)(recurse_to_match, child) {
                        Ok((true, did_inject)) => return Ok((true, did_inject)),
                        _ => (),
                    };
                }
                return Ok((false, false));
            }

            let did_inject = inject_html_data_attrs::<S>(
                data,
                m.expect("Document Node is already known to be Some"),
            )?;
            Ok((true, did_inject))
        },
    };

    match (recurse_to_match.f)(&recurse_to_match, node) {
        Ok((true, did_inject)) => Ok((true, did_inject)),
        Ok((false, _)) => Err(Error::NoBodyElementError),
        Err(e) => Err(e),
    }
}

fn inject_html_data_attrs<S: BuildHasher>(
    data: &HashMap<String, String, S>,
    element: &Rc<Node>,
) -> Result<bool, Error> {
    let NodeData::Element {
        name: qual_name,
        attrs,
        ..
    } = &element.data
    else {
        return Err(Error::ElementExpected(format!("{:?}", &element.data)));
    };

    let mut keys: Vec<String> = data
        .keys()
        .filter(|k| k.starts_with("PUBLIC_") || k.starts_with("public_"))
        .cloned()
        .collect();
    keys.sort_unstable();

    if keys.is_empty() {
        return Ok(false);
    }

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
        attrs.borrow_mut().push(new_attr);
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use crate::parse_html_and_inject_data;
    use std::collections::HashMap;

    #[test]
    fn parse_html_and_inject_data_sets_public_data() {
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

        match parse_html_and_inject_data(&data, html) {
            Ok((true, result_value)) => assert_eq!(&result_value, expected_html),
            Ok((false, _)) => panic!("should have returned 'true' that inject was successful"),
            Err(e) => panic!("returned error {e:?}"),
        }
    }

    #[test]
    fn parse_html_and_inject_data_corrects_invalid_doc() {
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
        let html = "<html><head><title>Hello World</title></head><h1>Hello World</h1></html>";
        let expected_html = r#"<html><head><title>Hello World</title></head><body data-public_api_url="https://api.example.com/v1" data-public_release_version="v101"><h1>Hello World</h1></body></html>"#;

        match parse_html_and_inject_data(&data, html) {
            Ok((true, result_value)) => assert_eq!(&result_value, expected_html),
            Ok((false, _)) => panic!("should have returned 'true' that inject was successful"),
            Err(e) => panic!("returned error {e:?}"),
        }
    }

    #[test]
    fn parse_html_and_inject_data_without_public_data_makes_no_diff() {
        let mut data: HashMap<String, String> = HashMap::new();
        data.insert(
            "NOT_PUBLIC_VAR".to_string(),
            "non-public should not be included".to_string(),
        );
        let html =
            "<html><head><title>Hello World</title></head><body><h1>Hello World</h1></body></html>";

        match parse_html_and_inject_data(&data, html) {
            Ok((false, _)) => (),
            Ok((true, _)) => panic!("should have returned 'false' that inject did not happen"),
            Err(e) => panic!("returned error {e:?}"),
        }
    }
}
