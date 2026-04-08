# frozen_string_literal: true

module Liquid
  class Tokenizer
    attr_reader :line_number, :for_liquid_tag

    TAG_END = /%\}/
    TAG_OR_VARIABLE_START = /\{[\{\%]/

    OPEN_CURLY = "{".ord
    CLOSE_CURLY = "}".ord
    PERCENT = "%".ord

    def initialize(source:, string_scanner:, line_numbers: false, line_number: nil, for_liquid_tag: false)
      @line_number = line_number || (line_numbers ? 1 : nil)
      @for_liquid_tag = for_liquid_tag
      @source = source.to_s.to_str
      @offset = 0
      @tokens = []

      return if @source.empty?

      @scanner = string_scanner
      @scanner.string = @source
      tokenize
    end

    def shift
      token = @tokens[@offset]
      return unless token

      @offset += 1
      @line_number += @for_liquid_tag ? 1 : token.count("\n") if @line_number
      token
    end

    private

    def tokenize
      @tokens =
        if @for_liquid_tag
          @source.split("\n")
        else
          tokens = []
          tokens << next_token until @scanner.eos?
          tokens
        end

      @source = nil
      @scanner = nil
    end

    def next_token
      byte_a = @scanner.peek_byte
      if byte_a == OPEN_CURLY
        @scanner.scan_byte
        byte_b = @scanner.peek_byte

        if byte_b == PERCENT
          @scanner.scan_byte
          return next_tag_token
        elsif byte_b == OPEN_CURLY
          @scanner.scan_byte
          return next_variable_token
        end

        @scanner.pos -= 1
      end

      next_text_token
    end

    def next_text_token
      start = @scanner.pos

      unless @scanner.skip_until(TAG_OR_VARIABLE_START)
        token = @scanner.rest
        @scanner.terminate
        return token
      end

      pos = @scanner.pos -= 2
      @source.byteslice(start, pos - start)
    rescue ::ArgumentError => error
      raise unless error.message.include?("invalid byte sequence")

      raise SyntaxError, "Invalid byte sequence in #{@scanner.string.encoding}"
    end

    def next_variable_token
      start = @scanner.pos - 2
      byte_a = byte_b = @scanner.scan_byte

      while byte_b
        byte_a = @scanner.scan_byte while byte_a && byte_a != CLOSE_CURLY && byte_a != OPEN_CURLY
        break unless byte_a

        if @scanner.eos?
          return byte_a == CLOSE_CURLY ? @source.byteslice(start, @scanner.pos - start) : "{{"
        end

        byte_b = @scanner.scan_byte

        if byte_a == CLOSE_CURLY
          if byte_b == CLOSE_CURLY
            return @source.byteslice(start, @scanner.pos - start)
          elsif byte_b != CLOSE_CURLY
            @scanner.pos -= 1
            return @source.byteslice(start, @scanner.pos - start)
          end
        elsif byte_a == OPEN_CURLY && byte_b == PERCENT
          return next_tag_token_with_start(start)
        end

        byte_a = byte_b
      end

      "{{"
    end

    def next_tag_token
      start = @scanner.pos - 2
      if (len = @scanner.skip_until(TAG_END))
        @source.byteslice(start, len + 2)
      else
        "{%"
      end
    end

    def next_tag_token_with_start(start)
      if @scanner.skip_until(TAG_END)
        @source.byteslice(start, @scanner.pos - start)
      else
        "{%"
      end
    end
  end
end
