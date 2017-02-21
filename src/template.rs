use Renderable;
use context::Context;
use filters::abs;
use filters::append;
use filters::capitalize;
use filters::ceil;
use filters::date;
use filters::divided_by;
use filters::downcase;
use filters::escape;
use filters::escape_once;
use filters::first;
use filters::floor;
use filters::join;
use filters::last;
use filters::lstrip;
use filters::minus;
use filters::modulo;
use filters::newline_to_br;
use filters::pluralize;
use filters::plus;
use filters::prepend;
use filters::remove;
use filters::remove_first;
use filters::replace;
use filters::replace_first;
use filters::reverse;
use filters::round;
use filters::rstrip;
use filters::size;
use filters::slice;
use filters::sort;
use filters::split;
use filters::strip;
use filters::strip_html;
use filters::strip_newlines;
use filters::times;
use filters::truncate;
use filters::truncatewords;
use filters::uniq;
use filters::upcase;
use error::Result;

pub struct Template {
    pub elements: Vec<Box<Renderable>>,
}

impl Renderable for Template {
    fn render(&self, context: &mut Context) -> Result<Option<String>> {

        context.maybe_add_filter("abs", Box::new(abs));
        context.maybe_add_filter("append", Box::new(append));
        context.maybe_add_filter("capitalize", Box::new(capitalize));
        context.maybe_add_filter("ceil", Box::new(ceil));
        context.maybe_add_filter("date", Box::new(date));
        context.maybe_add_filter("divided_by", Box::new(divided_by));
        context.maybe_add_filter("downcase", Box::new(downcase));
        context.maybe_add_filter("escape", Box::new(escape));
        context.maybe_add_filter("escape_once", Box::new(escape_once));
        context.maybe_add_filter("first", Box::new(first));
        context.maybe_add_filter("floor", Box::new(floor));
        context.maybe_add_filter("join", Box::new(join));
        context.maybe_add_filter("last", Box::new(last));
        context.maybe_add_filter("lstrip", Box::new(lstrip));
        context.maybe_add_filter("minus", Box::new(minus));
        context.maybe_add_filter("modulo", Box::new(modulo));
        context.maybe_add_filter("newline_to_br", Box::new(newline_to_br));
        context.maybe_add_filter("pluralize", Box::new(pluralize));
        context.maybe_add_filter("plus", Box::new(plus));
        context.maybe_add_filter("prepend", Box::new(prepend));
        context.maybe_add_filter("remove", Box::new(remove));
        context.maybe_add_filter("remove_first", Box::new(remove_first));
        context.maybe_add_filter("replace", Box::new(replace));
        context.maybe_add_filter("replace_first", Box::new(replace_first));
        context.maybe_add_filter("reverse", Box::new(reverse));
        context.maybe_add_filter("round", Box::new(round));
        context.maybe_add_filter("rstrip", Box::new(rstrip));
        context.maybe_add_filter("size", Box::new(size));
        context.maybe_add_filter("slice", Box::new(slice));
        context.maybe_add_filter("sort", Box::new(sort));
        context.maybe_add_filter("split", Box::new(split));
        context.maybe_add_filter("strip", Box::new(strip));
        context.maybe_add_filter("strip_html", Box::new(strip_html));
        context.maybe_add_filter("strip_newlines", Box::new(strip_newlines));
        context.maybe_add_filter("times", Box::new(times));
        context.maybe_add_filter("truncate", Box::new(truncate));
        context.maybe_add_filter("truncatewords", Box::new(truncatewords));
        context.maybe_add_filter("uniq", Box::new(uniq));
        context.maybe_add_filter("upcase", Box::new(upcase));

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
