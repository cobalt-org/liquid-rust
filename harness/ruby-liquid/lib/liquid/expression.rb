# frozen_string_literal: true

module Liquid
  class Expression
    LITERALS = {
      nil => nil,
      "nil" => nil,
      "null" => nil,
      "" => nil,
      "true" => true,
      "false" => false,
      "blank" => "",
      "empty" => "",
      "-" => VariableLookup.parse("-", nil).freeze,
    }.freeze

    DOT = ".".ord
    ZERO = "0".ord
    NINE = "9".ord
    DASH = "-".ord

    RANGES_REGEX = /\A\(\s*(?>(\S+)\s*\.\.)\s*(\S+)\s*\)\z/
    INTEGER_REGEX = /\A(-?\d+)\z/
    FLOAT_REGEX = /\A(-?\d+)\.\d+\z/

    class << self
      def safe_parse(parser, ss = StringScanner.new(""), cache = nil)
        parse(parser.expression, ss, cache)
      end

      def parse(markup, ss = StringScanner.new(""), cache = nil)
        return unless markup

        markup = markup.strip

        if (markup.start_with?("\"") && markup.end_with?("\"")) ||
            (markup.start_with?("'") && markup.end_with?("'"))
          return markup[1..-2]
        elsif LITERALS.key?(markup)
          return LITERALS[markup]
        end

        if cache
          return cache[markup] if cache.key?(markup)

          cache[markup] = inner_parse(markup, ss, cache).freeze
        else
          inner_parse(markup, ss, nil).freeze
        end
      end

      def inner_parse(markup, ss, cache)
        if markup.start_with?("(") && markup.end_with?(")") && markup =~ RANGES_REGEX
          return RangeLookup.parse(Regexp.last_match(1), Regexp.last_match(2), ss, cache)
        end

        parse_number(markup, ss) || VariableLookup.parse(markup, ss, cache)
      end

      def parse_number(markup, ss)
        case markup
        when INTEGER_REGEX
          return Integer(markup, 10)
        when FLOAT_REGEX
          return markup.to_f
        end

        ss.string = markup
        byte = ss.scan_byte
        return false if byte != DASH && (byte < ZERO || byte > NINE)

        if byte == DASH
          peek_byte = ss.peek_byte
          return false if peek_byte.nil? || !(peek_byte >= ZERO && peek_byte <= NINE)
        end

        saw_dot = false
        num_end_pos = nil

        while (byte = ss.scan_byte)
          return false if byte != DOT && (byte < ZERO || byte > NINE)
          next if num_end_pos

          if byte == DOT
            if saw_dot
              num_end_pos = ss.pos - 1
            else
              saw_dot = true
            end
          end
        end

        num_end_pos ||= markup.length
        markup.byteslice(0, num_end_pos).to_f
      end
    end
  end
end
