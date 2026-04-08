# frozen_string_literal: true

module Liquid
  class Block < Tag
    MAX_DEPTH = 100

    class CompiledSegment
      def initialize(template)
        @template = template
      end

      def render_to_output_buffer(context, output)
        @template.render_fragment_to_output_buffer(context, output)
      end
    end

    attr_reader :nodelist

    def initialize(tag_name, markup, parse_context)
      super
      @body = BlockBody.new([])
      @nodelist = @body.nodelist
      @render_segments = []
      @blank = true
    end

    def render(context)
      return @body.render(context) if @render_segments.empty?

      @render_segments.each_with_object(+"") do |segment, output|
        segment.render_to_output_buffer(context, output)
        break output if context&.interrupt?
      end
    end

    def parse(tokens)
      return unless tokens

      current_source = +""
      current_line = tokens.line_number
      nesting = []

      while (token = tokens.shift)
        tag = parse_tag_token(token)
        if tag
          tag_name = tag[:name]

          if nesting.empty? && top_level_control_tag?(tokens, tag_name)
            commit_body_source(tokens, current_source, current_line)
            current_source = +""
            current_line = tokens.line_number
            return nil if tag_name == block_delimiter

            unknown_tag(tag_name, tag[:markup], tokens)
            next
          end

          update_nesting(nesting, tag_name, tokens)
        end

        current_source << token
      end

      commit_body_source(tokens, current_source, current_line)
      nil
    end

    def unknown_tag(tag, _markup, _tokens)
      if tag == "else"
        raise Liquid::SyntaxError, "'else' is not a valid delimiter for #{block_name} tags. use #{block_delimiter}"
      elsif tag.start_with?("end")
        raise Liquid::SyntaxError, "'#{tag}' is not a valid delimiter for #{block_name} tags. use #{block_delimiter}"
      else
        raise Liquid::SyntaxError, "Unknown tag '#{tag}'"
      end
    end

    def blank?
      @blank
    end

    def body_template=(template)
      @body_template = template
      @body = template.root
      @nodelist = @body.nodelist
      @blank = @body.blank?
    end

    private

    TAG_TOKEN = /\A\{%-?\s*(#{Liquid::TagName})\s*(.*?)\s*-?%\}\z/m

    def self.raise_unknown_tag(tag, block_name, block_delimiter, parse_context)
      if tag == "else"
        raise SyntaxError, parse_context.locale.t("errors.syntax.unexpected_else", block_name: block_name)
      elsif tag.start_with?("end")
        raise SyntaxError, parse_context.locale.t(
          "errors.syntax.invalid_delimiter",
          tag: tag,
          block_name: block_name,
          block_delimiter: block_delimiter
        )
      else
        raise SyntaxError, parse_context.locale.t("errors.syntax.unknown_tag", tag: tag)
      end
    end

    def parse_tag_token(token)
      match = TAG_TOKEN.match(token)
      return nil unless match

      { name: match[1], markup: match[2].to_s }
    end

    def block_name
      @tag_name
    end

    def block_delimiter
      @block_delimiter ||= "end#{block_name}"
    end

    def raise_tag_never_closed(block_name)
      raise SyntaxError, parse_context.locale.t("errors.syntax.tag_never_closed", block_name: block_name)
    end

    def new_body
      parse_context.new_block_body
    end

    def parse_body(body, tokens)
      if parse_context.depth >= MAX_DEPTH
        raise StackLevelError, "Nesting too deep"
      end

      parse_context.depth += 1
      begin
        nodes, end_tag_name, end_tag_markup = Liquid::AstTemplateRoot.parse_nodes(tokens, parse_context)
        body.nodelist.concat(nodes)
        @blank &&= body.blank?

        return false if end_tag_name == block_delimiter
        raise_tag_never_closed(block_name) unless end_tag_name

        unknown_tag(end_tag_name, end_tag_markup, tokens)
      ensure
        parse_context.depth -= 1
      end

      true
    end

    def top_level_control_tag?(tokens, tag_name)
      tag_name.start_with?("end") || !registered_tag?(tokens, tag_name)
    end

    def registered_tag?(tokens, tag_name)
      tokens.environment&.tag_for_name(tag_name)
    end

    def update_nesting(nesting, tag_name, tokens)
      if tag_name.start_with?("end")
        closing_name = tag_name.delete_prefix("end")
        nesting.pop if nesting.last == closing_name
      elsif tokens.block_tag_names.include?(tag_name)
        nesting << tag_name
      end
    end

    def commit_body_source(tokens, source, line_number)
      return if source.empty?

      previous_line_number = tokens.line_number
      tokens.instance_variable_set(:@line_number, line_number)
      append_body_nodes(tokens.parse_body_nodelist(source, line_number: line_number))
      append_render_segment(tokens.compile_template(source))
    ensure
      tokens.instance_variable_set(:@line_number, previous_line_number)
    end

    def append_body_nodes(nodes)
      @body.nodelist.concat(nodes)
      @nodelist = @body.nodelist
      @blank &&= nodes.all? do |node|
        node.respond_to?(:blank?) ? node.blank? : node.to_s.match?(BlockBody::WhitespaceOrNothing)
      end
    end

    def append_render_segment(template)
      @render_segments << CompiledSegment.new(template)
    end
  end
end
