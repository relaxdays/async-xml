use crate::Error;
use quick_xml::events::Event;
use quick_xml::reader::Decoder;
use quick_xml::AsyncReader;
use tokio::io::AsyncBufRead;

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

impl<B, T> FromXml<B> for Option<T>
where
    B: AsyncBufRead + Send + Unpin,
    T: FromXml<B>,
{
    type Visitor = OptionalVisitor<T, B>;
}

pub struct OptionalVisitor<T, B>
where
    B: AsyncBufRead + Send + Unpin,
    T: FromXml<B>,
{
    empty: bool,
    inner_visitor: T::Visitor,
}

impl<T, B> Default for OptionalVisitor<T, B>
where
    B: AsyncBufRead + Send + Unpin,
    T: FromXml<B>,
{
    fn default() -> Self {
        Self {
            empty: true,
            inner_visitor: T::Visitor::default(),
        }
    }
}

#[async_trait::async_trait]
impl<B, T> Visitor<B> for OptionalVisitor<T, B>
where
    B: AsyncBufRead + Send + Unpin,
    T: FromXml<B>,
{
    type Output = Option<T>;

    fn start_name() -> Option<&'static str> {
        T::Visitor::start_name()
    }

    fn visit_attribute(&mut self, name: &str, value: &str) -> Result<(), Error> {
        self.empty = false;
        self.inner_visitor.visit_attribute(name, value)
    }

    async fn visit_child(
        &mut self,
        name: &str,
        reader: &mut PeekingReader<B>,
    ) -> Result<(), Error> {
        self.empty = false;
        self.inner_visitor.visit_child(name, reader).await
    }

    fn visit_text(&mut self, text: &str) -> Result<(), Error> {
        self.empty = false;
        self.inner_visitor.visit_text(text)
    }

    fn build(self) -> Result<Self::Output, Error> {
        if self.empty {
            return Ok(None);
        }
        let result = self.inner_visitor.build()?;
        Ok(Some(result))
    }
}
