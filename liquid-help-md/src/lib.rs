pub trait Renderable {
    fn render_summary(
        &self,
        stream: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>>;

    fn render(&self, stream: &mut dyn std::io::Write) -> Result<(), Box<dyn std::error::Error>>;
}

impl Renderable for liquid::compiler::FilterReflection {
    fn render_summary(
        &self,
        stream: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(stream, "**{}**: {}", self.name(), self.description())?;
        Ok(())
    }

    fn render(&self, stream: &mut dyn std::io::Write) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(stream, "**{}**: {}", self.name(), self.description())?;
        writeln!(stream, "")?;
        let params = self.positional_parameters();
        if !params.is_empty() {
            writeln!(stream, "Parameters (positional):")?;
            writeln!(stream, "| Name | Description | Required? |")?;
            writeln!(stream, "|------|-------------|-----------|")?;
            for param in params {
                writeln!(
                    stream,
                    "| {} | {} | {} |",
                    param.name,
                    param.description,
                    if param.is_optional { "no" } else { "yes" }
                )?;
            }
            writeln!(stream, "")?;
        }
        let params = self.keyword_parameters();
        if !params.is_empty() {
            writeln!(stream, "Parameters (named):")?;
            writeln!(stream, "| Name | Description | Required? |")?;
            writeln!(stream, "|------|-------------|-----------|")?;
            for param in params {
                writeln!(
                    stream,
                    "| {} | {} | {} |",
                    param.name,
                    param.description,
                    if param.is_optional { "no" } else { "yes" }
                )?;
            }
            writeln!(stream, "")?;
        }
        Ok(())
    }
}

impl Renderable for liquid::compiler::TagReflection {
    fn render_summary(
        &self,
        stream: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(stream, "**{}**: {}", self.tag(), self.description())?;
        Ok(())
    }

    fn render(&self, stream: &mut dyn std::io::Write) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(stream, "**{}**: {}", self.tag(), self.description())?;
        writeln!(stream, "")?;
        if let Some(spec) = self.spec() {
            writeln!(stream, "Grammar: `{}`", spec)?;
            writeln!(stream, "")?;
        }
        if let Some(example) = self.example() {
            writeln!(stream, "Example:")?;
            writeln!(stream, "```liquid")?;
            writeln!(stream, "{}", example)?;
            writeln!(stream, "```")?;
        }
        Ok(())
    }
}

impl Renderable for liquid::compiler::BlockReflection {
    fn render_summary(
        &self,
        stream: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(stream, "**{}**: {}", self.start_tag(), self.description())?;
        Ok(())
    }

    fn render(&self, stream: &mut dyn std::io::Write) -> Result<(), Box<dyn std::error::Error>> {
        writeln!(stream, "**{}**: {}", self.start_tag(), self.description())?;
        writeln!(stream, "")?;
        if let Some(spec) = self.spec() {
            writeln!(stream, "Grammar: `{}`", spec)?;
            writeln!(stream, "")?;
        }
        if let Some(example) = self.example() {
            writeln!(stream, "Example:")?;
            writeln!(stream, "```liquid")?;
            writeln!(stream, "{}", example)?;
            writeln!(stream, "```")?;
        }
        Ok(())
    }
}
