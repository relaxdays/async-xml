# async-xml

![crates.io](https://img.shields.io/crates/v/async-xml)
![crates.io](https://img.shields.io/docsrs/async-xml)

A crate based on `tokio` and `quick-xml` for deserializing XML data asynchronously. Includes derive-macros for deserializing things.

## Example

```rust
use async_xml::from_str;
use async_xml_derive::FromXml;

#[tokio::main]
async fn main() {
    let report: Report = from_str(r#"<report id="b"><data>text</data></report>"#)
        .await
        .unwrap();
    println!("deserialized: {:?}", report);
	// prints "Report { id: "b", data: Some(ReportData { data: "text" }) }"
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(rename = "report")]
pub struct Report {
    #[async_xml(attribute)]
    pub id: String,
    #[async_xml(child)]
    pub data: Option<ReportData>,
}

#[derive(Debug, PartialEq, FromXml)]
#[async_xml(rename = "data")]
pub struct ReportData {
    #[async_xml(value)]
    pub data: String,
}
```

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-Apache-2.0) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted  for inclusion in the fork by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
