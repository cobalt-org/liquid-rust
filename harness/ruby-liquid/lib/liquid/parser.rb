# frozen_string_literal: true

module Liquid
  class Parser
    def initialize(input)
      ss = input.is_a?(StringScanner) ? input : StringScanner.new(input)
      @tokens = Lexer.tokenize(ss)
      @pointer = 0
    end

    def jump(point)
      @pointer = point
    end

    def consume(type = nil)
      token = @tokens[@pointer]
      if type && token[0] != type
        raise SyntaxError, "Expected #{type} but found #{@tokens[@pointer].first}"
      end

      @pointer += 1
      token[1]
    end

    def consume?(type)
      token = @tokens[@pointer]
      return false unless token && token[0] == type

      @pointer += 1
      token[1]
    end

    def id?(str)
      token = @tokens[@pointer]
      return false unless token && token[0] == :id && token[1] == str

      @pointer += 1
      token[1]
    end

    def look(type, ahead = 0)
      token = @tokens[@pointer + ahead]
      token && token[0] == type
    end

    def expression
      token = @tokens[@pointer]
      raise SyntaxError, "Unexpected end of input while parsing expression" if token.nil?

      case token[0]
      when :id
        str = consume.dup
        str << variable_lookups
      when :open_square
        str = consume.dup
        str << expression
        str << consume(:close_square)
        str << variable_lookups
      when :string, :number
        consume
      when :open_round
        consume
        first = expression
        consume(:dotdot)
        last = expression
        consume(:close_round)
        "(#{first}..#{last})"
      else
        raise SyntaxError, "#{token} is not a valid expression"
      end
    end

    def argument
      str = +""
      if look(:id) && look(:colon, 1)
        str << consume << consume << " "
      end

      str << expression
      str
    end

    def variable_lookups
      str = +""

      loop do
        if look(:open_square)
          str << consume
          str << expression
          str << consume(:close_square)
        elsif look(:dot)
          str << consume
          str << consume(:id)
        else
          break
        end
      end

      str
    end
  end
end
