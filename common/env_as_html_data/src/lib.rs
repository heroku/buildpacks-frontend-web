extern crate html5ever;
extern crate markup5ever_rcdom as rcdom;

mod errors;
use errors::Error;

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{collections::HashMap, hash::BuildHasher, rc::Rc, str::FromStr};

use html5ever::driver::ParseOpts;
use html5ever::serialize::SerializeOpts;
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{ns, parse_document, serialize, Attribute, LocalName, QualName};
use rcdom::{Handle, Node, NodeData, RcDom, SerializableHandle};

pub enum HtmlRewritten {
    Yes,
    No,
}

pub fn env_as_html_data<S: BuildHasher>(
    data: &HashMap<String, String, S>,
    file_path: &PathBuf,
) -> Result<HtmlRewritten, Error> {
    let mut html_file = File::options()
        .read(true)
        .open(file_path)
        .map_err(|e| Error::FileError(e, format!("Tried to open {}", file_path.display())))?;
    let mut buffer = Vec::new();
    html_file
        .read_to_end(&mut buffer)
        .map_err(|e| Error::FileError(e, format!("Tried to read {}", file_path.display())))?;

    match parse_html_and_inject_data(data, &buffer)? {
        HtmlChanged::Yes(html_result) => {
            let mut rewrite_html_file = File::options()
                .write(true)
                .truncate(true)
                .open(file_path)
                .map_err(|e| {
                    Error::FileError(
                        e,
                        format!("Tried to reopen for writing {}", file_path.display()),
                    )
                })?;
            rewrite_html_file
                .write_all(html_result.as_bytes())
                .map_err(|e| {
                    Error::FileError(e, format!("Tried to write {}", file_path.display()))
                })?;
            Ok(HtmlRewritten::Yes)
        }
        HtmlChanged::No => Ok(HtmlRewritten::No),
    }
}

pub(crate) enum HtmlChanged {
    Yes(String),
    No,
}

pub(crate) fn parse_html_and_inject_data<S: BuildHasher>(
    data: &HashMap<String, String, S>,
    html_bytes: &[u8],
) -> Result<HtmlChanged, Error> {
    let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            ..Default::default()
        },
        ..Default::default()
    };
    let html = std::str::from_utf8(html_bytes)
        .map_err(|e| Error::ParseError(format!("could not decode HTML as UTF-8 {e:?}")))?;
    let dom = parse_document(RcDom::default(), opts).one(html);

    if let DataInjected::No = match_html_body_and_inject_data(data, &dom.document)? {
        return Ok(HtmlChanged::No);
    }

    let document: SerializableHandle = dom.document.clone().into();

    let mut buf = Vec::new();
    serialize(&mut buf, &document, SerializeOpts::default())
        .map_err(|e| Error::SerializeError(e.to_string()))?;
    let output =
        std::str::from_utf8(buf.as_slice()).map_err(|e| Error::EncodeError(e.to_string()))?;
    Ok(HtmlChanged::Yes(output.to_string()))
}

pub(crate) enum HtmlMatched {
    Yes,
    No,
}

pub(crate) enum DataInjected {
    Yes,
    No,
}

#[allow(clippy::type_complexity)]
fn match_html_body_and_inject_data<S: BuildHasher>(
    data: &HashMap<String, String, S>,
    node: &Handle,
) -> Result<DataInjected, Error> {
    // Closure around the reference-counted HTML DOM document/nodes, to support recursing to find the body element
    struct RecurseToMatch<'r> {
        f: &'r dyn Fn(&RecurseToMatch, &Rc<Node>) -> Result<(HtmlMatched, DataInjected), Error>,
    }
    let recurse_to_match = RecurseToMatch {
        f: &|recurse_to_match: &RecurseToMatch,
             n: &Rc<Node>|
         -> Result<(HtmlMatched, DataInjected), Error> {
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
                    if let Ok((HtmlMatched::Yes, did_inject)) =
                        (recurse_to_match.f)(recurse_to_match, child)
                    {
                        return Ok((HtmlMatched::Yes, did_inject));
                    }
                }
                return Ok((HtmlMatched::No, DataInjected::No));
            }

            let did_inject = inject_html_data_attrs::<S>(
                data,
                m.expect("Document Node is already known to be Some"),
            )?;
            Ok((HtmlMatched::Yes, did_inject))
        },
    };

    match (recurse_to_match.f)(&recurse_to_match, node) {
        Ok((HtmlMatched::Yes, did_inject)) => Ok(did_inject),
        Ok((HtmlMatched::No, _)) => Err(Error::NoBodyElementError),
        Err(e) => Err(e),
    }
}

fn inject_html_data_attrs<S: BuildHasher>(
    data: &HashMap<String, String, S>,
    element: &Rc<Node>,
) -> Result<DataInjected, Error> {
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
        return Ok(DataInjected::No);
    }

    let mut attrs_borrow = attrs.borrow_mut();
    for k in &keys {
        let name = format_html_data_attr_name(k);
        let value =
            StrTendril::from_str(data[k].as_str()).expect("Data key is already known to exist");

        // Replace existing attribute value, or create a new attribute.
        if let Some(attr) = attrs_borrow.iter_mut().find(|a| a.name.local == name) {
            attr.value = value;
        } else {
            let new_attr = Attribute {
                name: QualName {
                    prefix: qual_name.prefix.clone(),
                    ns: ns!(),
                    local: LocalName::from(name),
                },
                value,
            };
            attrs_borrow.push(new_attr);
        }
    }

    Ok(DataInjected::Yes)
}

fn format_html_data_attr_name(name: &str) -> String {
    format!("data-{}", name.to_lowercase())
}

#[cfg(test)]
mod tests {
    use crate::{env_as_html_data, parse_html_and_inject_data, HtmlChanged, HtmlRewritten};
    use std::{
        collections::HashMap,
        fs::{self, File},
        io::{Read, Write},
        path::Path,
    };
    use uuid::Uuid;

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn env_as_html_data_writes_into_file() {
        let mut data: HashMap<String, String> = HashMap::new();
        data.insert(
            "PUBLIC_API_URL".to_string(),
            "https://api.example.com/v1".to_string(),
        );
        let html =
            "<html><head><title>Hello World</title></head><body><h1>Hello World</h1></body></html>";
        let expected_html = r#"<html><head><title>Hello World</title></head><body data-public_api_url="https://api.example.com/v1"><h1>Hello World</h1></body></html>"#;

        let unique = Uuid::new_v4();
        let test_dir = format!("env-as-html-data-test-{unique}");
        let test_path = Path::new(&test_dir);
        fs::create_dir_all(test_path).expect("test dir should be created");
        let index_html_path = test_path.join("index.html");
        File::create(&index_html_path)
            .and_then(|mut file| file.write_all(html.as_bytes()))
            .expect("HTML file shoud be written");

        let mut result_html = Vec::new();
        match env_as_html_data(&data, &index_html_path) {
            Ok(HtmlRewritten::Yes) => {
                let mut result_file =
                    File::open(index_html_path).expect("the test file should exist");
                result_file
                    .read_to_end(&mut result_html)
                    .expect("test fixture file should contain bytes");
            }
            Ok(HtmlRewritten::No) => assert!(false, "should have rewritten the HTML"),
            Err(e) => assert!(false, "returned error {e:?}"),
        }

        fs::remove_dir_all(test_path).unwrap_or_default();

        assert_eq!(
            str::from_utf8(&result_html).expect("HTML contains valid UTF8 characters"),
            expected_html
        );
    }

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

        match parse_html_and_inject_data(&data, html.as_bytes()) {
            Ok(HtmlChanged::Yes(result_value)) => assert_eq!(&result_value, expected_html),
            Ok(HtmlChanged::No) => panic!("should have changed the HTML"),
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

        match parse_html_and_inject_data(&data, html.as_bytes()) {
            Ok(HtmlChanged::Yes(result_value)) => assert_eq!(&result_value, expected_html),
            Ok(HtmlChanged::No) => panic!("should have returned 'true' that inject was successful"),
            Err(e) => panic!("returned error {e:?}"),
        }
    }

    #[test]
    fn parse_html_and_inject_data_overwrites_existing_attrs() {
        let mut data: HashMap<String, String> = HashMap::new();
        data.insert(
            "PUBLIC_API_URL".to_string(),
            "https://api.example.com/v1".to_string(),
        );
        data.insert("PUBLIC_DEBUG_MODE".to_string(), "true".to_string());
        data.insert("PUBLIC_RELEASE_VERSION".to_string(), "v101".to_string());
        let html = r#"<html><head><title>Hello World</title></head><body data-public_api_url="http://localhost:3001/v1" data-public_release_version="v0"><h1>Hello World</h1></body></html>"#;
        let expected_html = r#"<html><head><title>Hello World</title></head><body data-public_api_url="https://api.example.com/v1" data-public_release_version="v101" data-public_debug_mode="true"><h1>Hello World</h1></body></html>"#;

        match parse_html_and_inject_data(&data, html.as_bytes()) {
            Ok(HtmlChanged::Yes(result_value)) => assert_eq!(&result_value, expected_html),
            Ok(HtmlChanged::No) => panic!("should have returned 'true' that inject was successful"),
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

        match parse_html_and_inject_data(&data, html.as_bytes()) {
            Ok(HtmlChanged::No) => (),
            Ok(HtmlChanged::Yes(_)) => {
                panic!("should have returned 'false' that inject did not happen")
            }
            Err(e) => panic!("returned error {e:?}"),
        }
    }
}
