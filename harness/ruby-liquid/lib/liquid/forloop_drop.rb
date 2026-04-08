# frozen_string_literal: true

module Liquid
  class ForloopDrop < Drop
    def initialize(name, length, parentloop)
      @name = name
      @length = length
      @parentloop = parentloop
      @index = 0
    end

    attr_reader :length, :parentloop, :name

    def index
      @index + 1
    end

    def index0
      @index
    end

    def rindex
      @length - @index
    end

    def rindex0
      @length - @index - 1
    end

    def first
      @index == 0
    end

    def last
      @index == @length - 1
    end

    def self.from_snapshot(snapshot)
      return snapshot unless snapshot.is_a?(Hash)

      parentloop = from_snapshot(snapshot["parentloop"])
      loop_drop = new(snapshot["name"], snapshot["length"], parentloop)
      index =
        if snapshot.key?("index0")
          snapshot["index0"]
        elsif snapshot.key?("index")
          snapshot["index"].to_i - 1
        else
          0
        end
      loop_drop.instance_variable_set(:@index, index)
      loop_drop
    end

    protected

    def increment!
      @index += 1
    end
  end
end
