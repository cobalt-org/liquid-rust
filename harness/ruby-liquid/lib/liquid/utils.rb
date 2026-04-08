# frozen_string_literal: true

module Liquid
  module Utils
    DECIMAL_REGEX = /\A-?\d+\.\d+\z/
    UNIX_TIMESTAMP_REGEX = /\A\d+\z/

    module_function

    def slice_collection(collection, from, to)
      if (from != 0 || !to.nil?) && collection.respond_to?(:load_slice)
        collection.load_slice(from, to)
      else
        slice_collection_using_each(collection, from, to)
      end
    end

    def slice_collection_using_each(collection, from, to)
      segments = []
      index = 0

      return collection.empty? ? [] : [collection] if collection.is_a?(String)
      return [] unless collection.respond_to?(:each)

      collection.each do |item|
        break if to && to <= index

        segments << item if from <= index
        index += 1
      end

      segments
    end

    def to_integer(num)
      return num if num.is_a?(Integer)
      num = num.to_s
      Integer(num)
    rescue ::ArgumentError
      raise Liquid::ArgumentError, "invalid integer"
    end

    def to_number(obj)
      case obj
      when Float
        BigDecimal(obj.to_s)
      when Numeric
        obj
      when String
        DECIMAL_REGEX.match?(obj.strip) ? BigDecimal(obj) : obj.to_i
      else
        obj.respond_to?(:to_number) ? obj.to_number : 0
      end
    end

    def to_date(obj)
      return obj if obj.respond_to?(:strftime)

      if obj.is_a?(String)
        return if obj.empty?

        obj = obj.downcase
      end

      case obj
      when "now", "today"
        Time.now
      when UNIX_TIMESTAMP_REGEX, Integer
        Time.at(obj.to_i)
      when String
        Time.parse(obj)
      end
    rescue ::ArgumentError
      nil
    end

    def to_liquid_value(obj)
      return obj.to_liquid_value if obj.respond_to?(:to_liquid_value)
      return obj.to_liquid if obj.respond_to?(:to_liquid)

      obj
    end

    def to_s(obj, seen = {})
      case obj
      when BigDecimal
        obj.to_s("F")
      when Hash
        if obj.class.instance_method(:to_s) == HASH_TO_S_METHOD
          hash_inspect(obj, seen)
        else
          obj.to_s
        end
      when Array
        array_inspect(obj, seen)
      else
        obj.to_s
      end
    end

    def inspect(obj, seen = {})
      case obj
      when Hash
        if obj.class.instance_method(:inspect) == HASH_INSPECT_METHOD
          hash_inspect(obj, seen)
        else
          obj.inspect
        end
      when Array
        array_inspect(obj, seen)
      else
        obj.inspect
      end
    end

    def array_inspect(arr, seen = {})
      return "[...]" if seen[arr.object_id]

      seen[arr.object_id] = true
      str = +"["
      cursor = 0
      len = arr.length

      while cursor < len
        str << ", " if cursor.positive?
        str << inspect(arr[cursor], seen)
        cursor += 1
      end

      str << "]"
      str
    ensure
      seen.delete(arr.object_id)
    end

    def hash_inspect(hash, seen = {})
      return "{...}" if seen[hash.object_id]

      seen[hash.object_id] = true
      str = +"{"
      first = true
      hash.each do |key, value|
        if first
          first = false
        else
          str << ", "
        end

        str << inspect(key, seen)
        str << "=>"
        str << inspect(value, seen)
      end
      str << "}"
      str
    ensure
      seen.delete(hash.object_id)
    end

    HASH_TO_S_METHOD = Hash.instance_method(:to_s)
    private_constant :HASH_TO_S_METHOD

    HASH_INSPECT_METHOD = Hash.instance_method(:inspect)
    private_constant :HASH_INSPECT_METHOD
  end
end
