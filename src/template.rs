use Renderable;
use context::Context;
use filters;
use error::Result;

pub struct Template {
    pub elements: Vec<Box<Renderable>>,
}

impl Renderable for Template {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {

        context.maybe_add_filter("abs", Box::new(filters::abs));
        context.maybe_add_filter("append", Box::new(filters::append));
        context.maybe_add_filter("capitalize", Box::new(filters::capitalize));
        context.maybe_add_filter("ceil", Box::new(filters::ceil));
        context.maybe_add_filter("compact", Box::new(filters::compact));
        context.maybe_add_filter("concat", Box::new(filters::concat));
        context.maybe_add_filter("date", Box::new(filters::date));
        context.maybe_add_filter("default", Box::new(filters::default));
        context.maybe_add_filter("divided_by", Box::new(filters::divided_by));
        context.maybe_add_filter("downcase", Box::new(filters::downcase));
        context.maybe_add_filter("escape", Box::new(filters::escape));
        context.maybe_add_filter("escape_once", Box::new(filters::escape_once));
        context.maybe_add_filter("first", Box::new(filters::first));
        context.maybe_add_filter("floor", Box::new(filters::floor));
        context.maybe_add_filter("join", Box::new(filters::join));
        context.maybe_add_filter("last", Box::new(filters::last));
        context.maybe_add_filter("lstrip", Box::new(filters::lstrip));
        context.maybe_add_filter("map", Box::new(filters::map));
        context.maybe_add_filter("minus", Box::new(filters::minus));
        context.maybe_add_filter("modulo", Box::new(filters::modulo));
        context.maybe_add_filter("newline_to_br", Box::new(filters::newline_to_br));
        context.maybe_add_filter("plus", Box::new(filters::plus));
        context.maybe_add_filter("prepend", Box::new(filters::prepend));
        context.maybe_add_filter("remove", Box::new(filters::remove));
        context.maybe_add_filter("remove_first", Box::new(filters::remove_first));
        context.maybe_add_filter("replace", Box::new(filters::replace));
        context.maybe_add_filter("replace_first", Box::new(filters::replace_first));
        context.maybe_add_filter("reverse", Box::new(filters::reverse));
        context.maybe_add_filter("round", Box::new(filters::round));
        context.maybe_add_filter("rstrip", Box::new(filters::rstrip));
        context.maybe_add_filter("size", Box::new(filters::size));
        context.maybe_add_filter("slice", Box::new(filters::slice));
        context.maybe_add_filter("sort", Box::new(filters::sort));
        context.maybe_add_filter("sort_natural", Box::new(filters::sort_natural));
        context.maybe_add_filter("split", Box::new(filters::split));
        context.maybe_add_filter("strip", Box::new(filters::strip));
        context.maybe_add_filter("strip_html", Box::new(filters::strip_html));
        context.maybe_add_filter("strip_newlines", Box::new(filters::strip_newlines));
        context.maybe_add_filter("times", Box::new(filters::times));
        context.maybe_add_filter("truncate", Box::new(filters::truncate));
        context.maybe_add_filter("truncatewords", Box::new(filters::truncatewords));
        context.maybe_add_filter("uniq", Box::new(filters::uniq));
        context.maybe_add_filter("upcase", Box::new(filters::upcase));
        context.maybe_add_filter("url_decode", Box::new(filters::url_decode));
        context.maybe_add_filter("url_encode", Box::new(filters::url_encode));

        #[cfg(feature = "extra-filters")]
        context.maybe_add_filter("pluralize", Box::new(filters::pluralize));
        #[cfg(feature = "extra-filters")]
        context.maybe_add_filter("date_in_tz", Box::new(filters::date_in_tz));

        let mut buf = String::new();
        for el in &self.elements {
            if let Some(ref x) = try!(el.render(context)) {
                buf = buf + x;
            }

            // Did the last element we processed set an interrupt? If so, we
            // need to abandon the rest of our child elements and just
            // return what we've got. This is usually in response to a
            // `break` or `continue` tag being rendered.
            if context.interrupted() {
                break;
            }
        }
        Ok(Some(buf))
    }
}

impl Template {
    pub fn new(elements: Vec<Box<Renderable>>) -> Template {
        Template { elements: elements }
    }
}
