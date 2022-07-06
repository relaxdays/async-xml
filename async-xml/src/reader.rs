use crate::Error;
use quick_xml::events::Event;
use quick_xml::reader::Decoder;
use quick_xml::AsyncReader;
use tokio::io::AsyncBufRead;

mod impls;

pub use impls::XmlFromStr;

pub struct PeekingReader<B: AsyncBufRead> {
    reader: AsyncReader<B>,
    peeked_event: Option<Event<'static>>,
}

impl<B: AsyncBufRead + Unpin + Send> PeekingReader<B> {
    pub fn from_buf(reader: B) -> Self {
        let mut reader = AsyncReader::from_reader(reader);
        Self::set_reader_defaults(&mut reader);
        Self {
            reader,
            peeked_event: None,
        }
    }

    pub fn into_inner(self) -> B {
        self.reader.into_underlying_reader()
    }

    fn set_reader_defaults(reader: &mut AsyncReader<B>) {
        reader.expand_empty_elements(true).trim_text(true);
    }

    pub async fn peek_event(&mut self) -> quick_xml::Result<&Event<'static>> {
        if self.peeked_event.is_none() {
            self.peeked_event = Some(self.next_event_internal().await?);
        }
        Ok(self.peeked_event.as_ref().unwrap())
    }

    pub async fn read_event(&mut self) -> quick_xml::Result<Event<'static>> {
        if let Some(event) = self.peeked_event.take() {
            return Ok(event);
        }
        self.next_event_internal().await
    }

    async fn next_event_internal(&mut self) -> quick_xml::Result<Event<'static>> {
        let mut buf = Vec::new();
        Ok(self.reader.read_event(&mut buf).await?.into_owned())
    }

    pub fn decoder(&self) -> Decoder {
        self.reader.decoder()
    }

    pub async fn skip_element(&mut self) -> Result<(), Error> {
        let dec = self.reader.decoder();
        let start_tag;
        match self.peek_event().await? {
            Event::Start(start) => {
                // check for start element name
                let name = start.local_name();
                let name = dec.decode(name);
                // store name to match expected end element
                start_tag = name.to_string();
                // remove peeked start event
                self.read_event().await?;
            }
            _ => {
                return Err(Error::MissingStart);
            }
        }
        let mut depth = 0_usize;

        loop {
            match self.peek_event().await? {
                Event::End(end) => {
                    let name = end.local_name();
                    let name = dec.decode(name).to_string();
                    // remove peeked end event
                    self.read_event().await?;
                    // check for name
                    if name == start_tag && depth == 0 {
                        return Ok(());
                    }
                    depth -= 1;
                }
                Event::Start(_) => {
                    self.read_event().await?;
                    depth += 1;
                }
                _ => {
                    self.read_event().await?;
                }
            }
        }
    }

    pub async fn deserialize<T>(&mut self) -> Result<T, Error>
    where
        T: FromXml<B>,
    {
        let mut visitor = T::Visitor::default();
        let dec = self.reader.decoder();

        let start_tag;
        match self.peek_event().await? {
            Event::Start(start) => {
                // check for start element name
                let name = start.local_name();
                let name = dec.decode(name);
                if let Some(expected_name) = T::Visitor::start_name() {
                    if name != expected_name {
                        return Err(Error::WrongStart(expected_name.into(), name.into()));
                    }
                }
                visitor.visit_tag(&name)?;
                // store name to match expected end element
                start_tag = name.to_string();
                // read attributes
                for attr in start.attributes() {
                    let attr = attr?;
                    let attr_name = dec.decode(attr.key);
                    let attr_value = attr.unescaped_value()?;
                    let attr_value = dec.decode(&attr_value);
                    visitor.visit_attribute(&attr_name, &attr_value)?;
                }
                // remove peeked start event
                self.read_event().await?;
            }
            _ => {
                return Err(Error::MissingStart);
            }
        }

        loop {
            match self.peek_event().await? {
                Event::End(end) => {
                    let name = end.local_name();
                    let name = dec.decode(name).to_string();
                    // remove peeked end event
                    self.read_event().await?;
                    // check for name
                    if name != start_tag {
                        return Err(Error::WrongEnd(start_tag, name));
                    }
                    return visitor.build();
                }
                Event::Text(text) => {
                    let text = text.unescaped()?;
                    let text = dec.decode(&text);
                    visitor.visit_text(&text)?;
                    // remove peeked event
                    self.read_event().await?;
                }
                Event::Start(start) => {
                    // peeked child start element -> find name and call into sub-element
                    let name = start.local_name();
                    let name = dec.decode(name).to_string();
                    visitor.visit_child(&name, self).await?;
                }
                _ => {
                    self.read_event().await?;
                }
            }
        }
    }
}

impl<'r> PeekingReader<&'r [u8]> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: &'r str) -> Self {
        Self::from_buf(str.as_bytes())
    }
}

pub trait FromXml<B: AsyncBufRead + Send + Unpin> {
    type Visitor: Visitor<B, Output = Self> + Default + Send;
}

#[async_trait::async_trait]
pub trait Visitor<B: AsyncBufRead + Send + Unpin> {
    type Output;

    fn start_name() -> Option<&'static str> {
        None
    }

    #[allow(unused_variables)]
    fn visit_tag(&mut self, name: &str) -> Result<(), Error> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn visit_attribute(&mut self, name: &str, value: &str) -> Result<(), Error> {
        Err(Error::UnexpectedAttribute(name.into()))
    }

    #[allow(unused_variables)]
    async fn visit_child(
        &mut self,
        name: &str,
        reader: &mut PeekingReader<B>,
    ) -> Result<(), Error> {
        Err(Error::UnexpectedChild(name.into()))
    }

    #[allow(unused_variables)]
    fn visit_text(&mut self, text: &str) -> Result<(), Error> {
        Err(Error::UnexpectedText)
    }

    fn build(self) -> Result<Self::Output, Error>;
}
