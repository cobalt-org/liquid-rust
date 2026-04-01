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

    def to_s(with_prefix: true)
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

    def self.prefix
      "Liquid error"
    end

    def self.wrap(exception, default_class: self)
      return exception if exception.is_a?(Liquid::Error)

      error_class, metadata = classify_exception(exception.message.to_s, default_class)
      error_class.new(
        metadata[:message],
        line_number: metadata[:line_number],
        template_name: metadata[:template_name],
        markup_context: metadata[:markup_context],
        cause: exception
      )
    end

    def self.classify_exception(message, default_class)
      metadata = extract_metadata(message)

      if message.include?("Unknown filter")
        requested = message[/requested filter=([^\n]+)/, 1]
        metadata[:message] = requested ? "undefined filter #{requested}" : "undefined filter"
        return [Liquid::UndefinedFilter, metadata]
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

    def self.extract_metadata(message)
      normalized = message.to_s.dup
      line_number = normalized[/-->\s+(\d+):\d+/, 1]&.to_i

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
      elsif (line = normalized.lines.find { |entry| entry.lstrip.start_with?("=") })
        normalized = line.sub(/^.*=\s*/, "").strip
      else
        normalized = normalized.sub(/\Aliquid:\s*/, "").strip
      end

      {
        message: normalized,
        line_number: line_number,
        template_name: nil,
        markup_context: nil,
      }
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
  end

  class SyntaxError < Error
    def self.prefix
      "Liquid syntax error"
    end
  end

  class ArgumentError < Error; end
  class FileSystemError < Error; end
  class ContextError < Error; end
  class StackLevelError < Error; end
  class MemoryError < Error; end
  class UndefinedVariable < Error; end
  class UndefinedDropMethod < Error; end
  class UndefinedFilter < Error; end
  class InternalError < Error; end
end
