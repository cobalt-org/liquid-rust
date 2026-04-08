# frozen_string_literal: true

module Liquid
  class Error < StandardError
    attr_accessor :line_number, :template_name, :markup_context

    def initialize(message = nil, line_number: nil, template_name: nil, markup_context: nil, cause: nil)
      @line_number = line_number
      @template_name = template_name
      @markup_context = markup_context
      @cause = cause
      super(message)
    end

    def to_s(with_prefix = true)
      base = super()
      return base unless with_prefix

      message = +""
      message << message_prefix
      message << base
      if markup_context
        message << " "
        message << markup_context
      end
      message
    end

    def cause
      @cause
    end

    def ==(other)
      return false unless other.is_a?(self.class)

      message == other.message &&
        line_number == other.line_number &&
        template_name == other.template_name &&
        markup_context == other.markup_context &&
        cause_signature(cause) == cause_signature(other.cause)
    end

    alias eql? ==

    def self.prefix
      "Liquid error"
    end

    def hash
      [
        self.class,
        message,
        line_number,
        template_name,
        markup_context,
        cause_signature(cause),
      ].hash
    end

    def self.wrap(exception, default_class: self, **options)
      return exception if exception.is_a?(Liquid::Error)

      error_class, metadata = classify_exception(exception.message.to_s, default_class, **options)
      error_class.new(
        metadata[:message],
        line_number: metadata[:line_number],
        template_name: metadata[:template_name],
        markup_context: metadata[:markup_context],
        cause: exception
      )
    end

    def self.classify_exception(message, default_class, **options)
      metadata = extract_metadata(message, **options)
      normalized_message = metadata[:message].to_s

      if message.include?("Unknown filter")
        requested = message[/requested filter=([^\n]+)/, 1]
        metadata[:message] = requested ? "undefined filter #{requested}" : "undefined filter"
        return [Liquid::UndefinedFilter, metadata]
      end

      if message.include?("Argument error in tag 'include' - Illegal template name")
        metadata[:message] = "Argument error in tag 'include' - Illegal template name"
        return [Liquid::ArgumentError, metadata]
      end

      if message.include?("wrong number of arguments") ||
          message.include?("Unexpected named argument") ||
          message.include?("Expected named argument") ||
          message.include?("Multiple definitions of `")
        metadata[:message] = metadata[:message].sub(/\Aliquid:\s*/, "")
        return [Liquid::ArgumentError, metadata]
      end

      if message.include?("Expected id but found end_of_string") || message.include?("Unexpected character")
        return [Liquid::SyntaxError, metadata]
      end

      if message.include?("stack level too deep")
        metadata[:message] = "stack level too deep"
        return [Liquid::StackLevelError, metadata]
      end

      if message.include?("Can't divide by zero") || normalized_message.include?("divided by 0")
        metadata[:message] = "divided by 0"
        return [Liquid::ZeroDivisionError, metadata]
      end

      if normalized_message.include?("Computation results in 'Infinity'")
        metadata[:message] = "Computation results in 'Infinity'"
        return [Liquid::FloatDomainError, metadata]
      end

      if normalized_message.include?("cannot select the property")
        return [Liquid::ArgumentError, metadata]
      end

      if normalized_message.include?("invalid integer")
        metadata[:message] = "invalid integer"
        return [Liquid::ArgumentError, metadata]
      end

      if message.include?("Memory limits exceeded")
        metadata[:message] = "Memory limits exceeded"
        return [Liquid::MemoryError, metadata]
      end

      if message.include?("Undefined drop method")
        requested = message[/requested variable=([^\n]+)/, 1]
        metadata[:message] = requested ? "undefined drop method #{requested}" : "undefined drop method"
        return [Liquid::UndefinedDropMethod, metadata]
      end

      if message.include?("Unknown variable")
        requested = message[/requested variable=([^\n]+)/, 1]
        metadata[:message] = requested ? "undefined variable #{requested}" : "undefined variable"
        return [Liquid::UndefinedVariable, metadata]
      end

      [default_class, metadata]
    end

    def self.extract_metadata(message, line_numbers: nil, source: nil, error_mode: nil)
      normalized = message.to_s.dup
      normalized.sub!(/\ALiquid(?: syntax)? error(?: \([^)]+\))?:\s*/, "")
      diagnostic_line_number = normalized[/-->\s+(\d+):\d+/, 1]&.to_i
      diagnostic_column = normalized[/-->\s+\d+:(\d+)/, 1]&.to_i
      source_line = diagnostic_line(source, diagnostic_line_number) || normalized.lines.find { |entry| entry.match?(/^\s*\d+\s+\|/) }&.sub(/^\s*\d+\s+\|\s?/, "")&.chomp
      detail = normalized.lines.find { |entry| entry.lstrip.start_with?("=") }&.sub(/^\s*=\s*/, "")&.strip
      line_number = line_numbers ? diagnostic_line_number : nil
      strict_mode = error_mode.nil? || error_mode.to_sym == :strict || error_mode.to_sym == :strict2

      if normalized.include?("Unknown tag.")
        requested = normalized[/requested=([^\n]+)/, 1]
        normalized =
          if requested == "else"
            "Unexpected outer 'else' tag"
          elsif requested
            "Unknown tag '#{requested}'"
          else
            "Unknown tag"
          end
      elsif normalized.include?("expected Identifier or Value")
        normalized = "is not a valid expression"
      elsif strict_mode && detail == "expected Identifier"
        normalized = "Expected id but found end_of_string in #{extract_output_snippet(source_line, diagnostic_column).inspect}"
      elsif normalized =~ /\Aunexpected "([^"]+)"; expected Identifier\z/
        normalized = "Unexpected character #{Regexp.last_match(1)}"
      elsif strict_mode && detail&.start_with?("expected Value")
        normalized = unexpected_character_message(source_line, diagnostic_column)
      elsif strict_mode && detail&.start_with?("Unclosed block.")
        normalized = detail.sub(/\AUnclosed block\. \{%\s*end(\w+)\s*%\} tag expected\.\z/, "'\\1' tag was never closed")
      elsif cycle_syntax_error?(normalized, source_line)
        normalized = "Syntax Error in 'cycle' - Valid syntax: cycle [name :] var [, var2, var3 ...]"
      elsif assign_syntax_error?(normalized, detail, source_line)
        normalized = "Syntax Error in 'assign' - Valid syntax: assign [var] = [source]"
      elsif strict_mode && assign_range_syntax_error?(normalized, detail, source_line)
        normalized = assign_range_syntax_error_message(source_line)
      elsif strict_mode && case_syntax_error?(normalized, detail, source_line)
        normalized = case_syntax_error_message(source_line)
      elsif detail
        normalized = detail
      else
        normalized = normalized.split("\nfrom:").first.to_s.sub(/\Aliquid:\s*/, "").strip
      end

      if normalized =~ /\Aunexpected "([^"]+)"; expected Identifier\z/
        normalized = "Unexpected character #{Regexp.last_match(1)}"
      end

      {
        message: normalized,
        line_number: line_number,
        template_name: nil,
        markup_context: nil,
      }
    end

    def self.diagnostic_line(source, line_number)
      return nil unless source && line_number

      source.to_s.lines[line_number - 1]&.chomp
    end

    def self.extract_output_snippet(source_line, column)
      return source_line.to_s unless source_line

      start_index = source_line.rindex("{{", column ? [column - 1, 0].max : source_line.length) || 0
      end_index = source_line.index("}}", column ? [column - 1, 0].max : 0)
      end_index = end_index ? end_index + 1 : source_line.length - 1
      source_line[start_index..end_index].to_s.strip
    end

    def self.extract_tag_expression(source_line, column)
      return source_line.to_s.strip unless source_line

      start_index = source_line.rindex("{%", column ? [column - 1, 0].max : source_line.length) || 0
      end_index = source_line.index("%}", column ? [column - 1, 0].max : 0)
      end_index = end_index ? end_index - 1 : source_line.length - 1
      content = source_line[(start_index + 2)..end_index].to_s.strip
      content.sub(/\A(if|elsif)\s+/, "")
    end

    def self.unexpected_character_message(source_line, column)
      return "\"" unless source_line && column && column > 0

      offending = source_line[column - 1]
      if offending == "!" && column > 1 && source_line[column - 2] == "="
        offending = "="
      end
      snippet =
        if source_line.include?("{{")
          extract_output_snippet(source_line, column)
        else
          extract_tag_expression(source_line, column)
        end

      %(Unexpected character #{offending} in #{snippet.inspect})
    end

    def self.assign_syntax_error?(normalized, detail, source_line)
      source_line&.include?("{%") &&
        source_line.include?("assign") &&
        [normalized, detail].compact.any? { |entry| entry.include?('Assignment operator "=" expected.') }
    end

    def self.cycle_syntax_error?(normalized, source_line)
      source_line&.include?("{%") &&
        source_line.include?("cycle") &&
        normalized.include?("Identifier or value expected")
    end

    def self.assign_range_syntax_error?(normalized, detail, source_line)
      return false unless source_line&.include?("{%") && source_line.include?("assign")

      [normalized, detail].compact.any? do |entry|
        entry.include?("expected NilLiteral, EmptyLiteral, BlankLiteral, or Range")
      end
    end

    def self.assign_range_syntax_error_message(source_line)
      expression = assign_range_expression(source_line)
      offending = expression&.include?("|") ? "pipe" : "token"
      snippet = expression ? "{{#{expression} }}" : extract_tag_expression(source_line, nil)
      "Expected dotdot but found #{offending} in #{snippet.inspect}"
    end

    def self.assign_range_expression(source_line)
      expression = extract_tag_expression(source_line, nil).sub(/\Aassign\s+[^\s]+\s*=\s*/, "")
      expression unless expression.empty?
    end

    def self.case_syntax_error?(normalized, detail, source_line)
      return false unless source_line&.include?("{%")
      return false unless source_line.include?("case") || source_line.include?("when")

      [normalized, detail].compact.any? do |entry|
        entry.include?("unexpected FilterChain") ||
          entry.include?('"or" or "," expected.') ||
          entry == 'unexpected "="'
      end
    end

    def self.case_syntax_error_message(source_line)
      expression = extract_tag_expression(source_line, nil)
      return "Unexpected character =" if expression.include?("=>")

      "Expected end_of_string but found trailing content in #{expression.inspect}"
    end

    private

    def message_prefix
      prefix = +""
      prefix << self.class.prefix
      if line_number
        prefix << " ("
        prefix << "#{template_name} " if template_name
        prefix << "line #{line_number}"
        prefix << ")"
      end
      prefix << ": "
      prefix
    end

    def cause_signature(error)
      return nil unless error

      [error.class, error.message]
    end
  end

  class SyntaxError < Error
    def self.prefix
      "Liquid syntax error"
    end
  end

  class StandardError < Error; end
  class ArgumentError < Error; end
  class FileSystemError < Error; end
  class ContextError < Error; end
  class StackLevelError < Error; end
  class MemoryError < Error; end
  class TemplateEncodingError < Error; end
  class ZeroDivisionError < Error; end
  class FloatDomainError < Error; end
  class UndefinedVariable < Error; end
  class UndefinedDropMethod < Error; end
  class UndefinedFilter < Error; end
  class InternalError < Error; end
  class DisabledError < Error; end
end
