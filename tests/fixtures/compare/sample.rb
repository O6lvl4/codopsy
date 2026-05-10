# TODO: refactor this module
require './lib/helper'

def process(items)
  items.each do |item|
    puts item
    p item
    eval("item * 2")
    sleep(1)
  end
rescue Exception => e
  puts e.message
end

def empty_method
end

def complex(a, b, c, d, e)
  a + b + c + d + e
end
