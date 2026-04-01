# frozen_string_literal: true

module Liquid
  class BlockBody
    attr_reader :nodelist

    def initialize(nodelist = [])
      @nodelist = Array(nodelist)
    end

    def self.from_native(native)
      new(native || [])
    end

    def blank?
      @nodelist.all? do |node|
        node.respond_to?(:blank?) ? node.blank? : node.to_s.empty?
      end
    end
  end
end
